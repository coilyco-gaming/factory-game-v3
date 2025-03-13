namespace Assets.Scripts.Components.Core
{
    using System.Collections.Generic;
    using System.Linq;
    using Assets.Scripts.Core;

    public class DispatchComponentCore
    {
        // Example descriptions:
        //  - Retrieve power lines from me
        //  - Deploy mining drill to iron ore
        //  - Deliver coal to me
        private string DescriptionToOrFrom =>
            this.receiverVerb == DispatchComponentCore.Verbs.Retrieve.ToString()
            || this.receiverVerb == DispatchComponentCore.Verbs.Stockpile.ToString()
                ? "from"
                : "to";
        private string DescriptionSubject => Util.HumanizedString(this.receiverSubject);
        private string DescriptionObject => Util.HumanizedString(this.receiverObject);
        public string Description =>
            $"{this.receiverVerb} {this.DescriptionSubject} {this.DescriptionToOrFrom} {this.DescriptionObject}".ToLower();

        public WorldObjectCore worldObject;
        public DispatchReceiverComponentCore receiver;
        private BatteryComponentCore battery;
        private ResourcesComponentCore resources;
        private GameContent gameContent;
        public string receiverVerb = "VERB";
        public string receiverSubject = "SUBJECT";
        public string receiverObject = "OBJECT";
        private Dictionary<string, List<string>> VerbMappings = new()
        {
            {
                // Deploy mining drill to iron ore dispatches to
                //   - Deploy mining drill (mobile receiver)
                Verbs.Deploy.ToString(),
                new List<string> { Verbs.Deploy.ToString() }
            },
            {
                // Deliver iron bars to me dispatches to
                //   - Deliver iron bars
                Verbs.Deliver.ToString(),
                new List<string> { Verbs.Deliver.ToString(), Verbs.Stockpile.ToString() }
            },
            {
                // Retrieve iron bars from me dispatches to
                //   - Retrieve iron bars
                //   - Stockpile iron bars
                Verbs.Retrieve.ToString(),
                new List<string> { Verbs.Retrieve.ToString(), Verbs.Stockpile.ToString() }
            },
            // Stockpile should never dispatch
        };

        public enum Verbs
        {
            Deploy,
            Deliver,
            Retrieve, // Get something once, used for truck receivers
            Stockpile, // Get something repeatedly, used for factory receivers
        }

        public enum Keywords
        {
            Me,
        }

        public DispatchComponentCore(
            WorldObjectCore worldObject,
            BatteryComponentCore battery,
            ResourcesComponentCore resources,
            GameContent gameContent,
            string receiverVerb,
            string receiverSubject,
            string receiverObject
        )
        {
            this.battery =
                battery
                ?? throw new GameControllerCore.MisconfigurationException(
                    "Dispatch component requires a battery component"
                );
            this.worldObject =
                worldObject
                ?? throw new GameControllerCore.MisconfigurationException(
                    "Dispatch component requires a parent world object"
                );
            this.resources =
                resources
                ?? throw new GameControllerCore.MisconfigurationException(
                    "Dispatch component requires a resources component"
                );
            this.gameContent = gameContent;
            this.receiverVerb = receiverVerb;
            this.receiverSubject = receiverSubject;
            this.receiverObject = receiverObject;
        }

        public List<Dictionary<uint, string>> Tick(GameControllerCore gameController)
        {
            // If the dispatch has already been assigned, then skip
            if (this.receiver != null)
            {
                return new();
            }

            // If the dispatch subject is an item,
            // then skip deliver to me dispatch if I have more than a stack of the item
            // TODO: check if you can't fit any more
            if (
                this.receiverVerb == DispatchComponentCore.Verbs.Deliver.ToString()
                && this.receiverObject == DispatchComponentCore.Keywords.Me.ToString()
                && this.resources.resources.GetValueOrDefault(this.receiverSubject)
                    >= this.gameContent.Items[this.receiverSubject].StackSize
            )
            {
                return new List<Dictionary<uint, string>>
                {
                    new()
                    {
                        {
                            gameController.backref.TickCount,
                            $"{this.Description}: no more required"
                        },
                    },
                };
            }

            // If the dispatch subject is an item,
            // then skip retrieve from me dispatch if I have less than a stack of the item
            if (
                this.receiverVerb == DispatchComponentCore.Verbs.Retrieve.ToString()
                && this.receiverObject == DispatchComponentCore.Keywords.Me.ToString()
                && this.resources.resources.GetValueOrDefault(this.receiverSubject)
                    < this.gameContent.Items[this.receiverSubject].StackSize
            )
            {
                return new List<Dictionary<uint, string>>
                {
                    new()
                    {
                        {
                            gameController.backref.TickCount,
                            $"{this.Description}: not enough available"
                        },
                    },
                };
            }

            // Only dispatch if there's an empty adjacent tile
            bool hasEmptyAdjacent = false;
            foreach (
                System.Numerics.Vector2 adjacentTile in GameControllerCore.GetAdjacentPositions(
                    this.worldObject.gridPosition
                )
            )
            {
                if (!gameController.worldObjects.ContainsKey(adjacentTile))
                {
                    hasEmptyAdjacent = true;
                    break;
                }
                if (
                    gameController.worldObjects.ContainsKey(adjacentTile)
                    && gameController.worldObjects[adjacentTile].Count == 0
                )
                {
                    hasEmptyAdjacent = true;
                    break;
                }
            }
            if (!hasEmptyAdjacent)
            {
                return new List<Dictionary<uint, string>>
                {
                    new()
                    {
                        {
                            gameController.backref.TickCount,
                            $"{this.Description}: no empty adjacent tile"
                        },
                    },
                };
            }

            // Abort early if the battery is empty
            try
            {
                this.battery.Energy -= 1;
            }
            catch (BatteryComponentCore.BatteryCapacityException)
            {
                return new();
            }

            // Acqiure list the target location
            List<System.Numerics.Vector2> targetLocations =
                this.receiverObject == DispatchComponentCore.Keywords.Me.ToString()
                    ? new List<System.Numerics.Vector2> { this.worldObject.gridPosition }
                    : gameController
                        .worldObjects
                        // For world object locations that do not already have a dispatch
                        .SelectMany(worldObjects => worldObjects.Value)
                        // For world objects that contain the target type
                        .Where(worldObject =>
                            worldObject.Value.worldObjectType == this.receiverObject
                        )
                        // Order by distance to the current world object
                        .OrderBy(worldObject =>
                            System.Numerics.Vector2.Distance(
                                worldObject.Value.GridPosition,
                                this.worldObject.GridPosition
                            )
                        )
                        // Select the grid position of the target world objects
                        .Select(worldObject => worldObject.Value.GridPosition)
                        .ToList();

            // TODO: don't assign is target is not adjacent or there is a path to the target

            if (targetLocations.Count == 0)
            {
                return new List<Dictionary<uint, string>>
                {
                    new()
                    {
                        {
                            gameController.backref.TickCount,
                            $"{this.Description}: no target found"
                        },
                    },
                };
            }

            // TODO: immobible recievers only match when they are adjacent
            // Get the first receiver awaiting a target
            DispatchReceiverComponentCore receiver = gameController
                .worldObjects
                // For all world objects
                .SelectMany(worldObjects => worldObjects.Value)
                .Where(worldObject =>
                    // Where the world object has dispatch receivers
                    worldObject.Value.dispatchReceivers != null
                )
                // For all dispatch receivers
                .SelectMany(worldObject => worldObject.Value.dispatchReceivers)
                .Where(receiver =>
                    // Where the receiver is not null and is awaiting a target
                    receiver != null
                    && receiver.dispatcher == null
                    // Where the receiver Subject and verb match the dispatch
                    && receiver.receiverSubject == this.receiverSubject
                    && this.VerbMappings[this.receiverVerb].Contains(receiver.receiverVerb)
                )
                // Order by distance to the current world object
                .OrderBy(receiver =>
                    System.Numerics.Vector2.Distance(
                        receiver.worldObject.GridPosition,
                        this.worldObject.GridPosition
                    )
                )
                .FirstOrDefault();

            if (receiver == null)
            {
                return new List<Dictionary<uint, string>>
                {
                    new()
                    {
                        {
                            gameController.backref.TickCount,
                            $"{this.Description}: no receiver found"
                        },
                    },
                };
            }

            // Assign the target to the receiver
            receiver.targetPosition = targetLocations[0];
            receiver.dispatcher = this;
            this.receiver = receiver;
            return new();
        }
    }
}

namespace Assets.Scripts.Components.Tests
{
    using System.Collections.Generic;
    using System.Linq;
    using Assets.Scripts.Components.Core;
    using Assets.Scripts.Core;
    using Assets.Scripts.Unity;
    using Xunit;
    using Xunit.Abstractions;

    internal class TestDispatchGameContent : GameContent
    {
        public override Dictionary<string, Item> Items { get; } =
            new()
            {
                {
                    "IronBars",
                    new Item(
                        "IronBars",
                        stackSize: 10,
                        ingredients: new Dictionary<string, uint> { { "IronOre", 5 } }
                    )
                },
                {
                    "MiningDrill",
                    new Item(
                        "MiningDrill",
                        stackSize: 1,
                        craftTime: 3,
                        ingredients: new Dictionary<string, uint> { { "IronBars", 5 } }
                    )
                },
            };
    }

    internal class TestDispatchUnityGameController : IGameController
    {
        public uint TickCount { get; set; } = 0;
    }

    public class DispatchComponentTest
    {
        private ITestOutputHelper testOutput;

        public DispatchComponentTest(ITestOutputHelper testOutputHelper)
        {
            this.testOutput = testOutputHelper;
        }

        [Fact]
        public void TestTrue()
        {
            GameControllerCore gameController = new()
            {
                backref = new TestDispatchUnityGameController(),
            };
            WorldObjectCore worldObject = new(null);
            BatteryComponentCore battery = new(100, 100);
            ResourcesComponentCore resources = new(new(), 100, 100);
            DispatchComponentCore dispatch = new(
                worldObject,
                battery,
                resources,
                new TestDispatchGameContent(),
                "",
                "",
                ""
            );
            dispatch.Tick(gameController);
            Assert.True(true);
        }

        [Fact]
        public void TestAssignTarget()
        {
            GameControllerCore gameController = new()
            {
                backref = new TestDispatchUnityGameController(),
            };
            WorldObjectCore HQWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(0, 0),
            };

            BatteryComponentCore battery = new(100, 100);
            ResourcesComponentCore dispactherResources = new(
                new TestDispatchGameContent(),
                100,
                100
            );
            DispatchComponentCore dispatch = new(
                HQWorldObject,
                battery,
                dispactherResources,
                new TestDispatchGameContent(),
                "Deploy",
                "MiningDrill",
                "IronOre"
            );
            HQWorldObject.dispatchers = new List<DispatchComponentCore> { dispatch };

            WorldObjectCore targetWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(1, 1),
                worldObjectType = "IronOre",
            };

            WorldObjectCore receiverWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(2, 2),
            };

            ResourcesComponentCore receiverResources = new(new(), 100, 100);

            DispatchReceiverComponentCore receiver = new(
                receiverWorldObject,
                receiverResources,
                "Deploy",
                "MiningDrill"
            );
            receiverWorldObject.dispatchReceivers = new() { receiver };
            Assert.Null(receiver.dispatcher);

            gameController.worldObjects[new System.Numerics.Vector2(0, 0)] = new()
            {
                { "uuid-1", HQWorldObject },
            };
            gameController.worldObjects[new System.Numerics.Vector2(1, 1)] = new()
            {
                { "uuid-2", targetWorldObject },
            };
            gameController.worldObjects[new System.Numerics.Vector2(2, 2)] = new()
            {
                { "uuid-3", receiverWorldObject },
            };

            dispatch.Tick(gameController);
            Assert.NotNull(receiver.dispatcher);
        }

        [Fact]
        public void TestVerbMapping()
        {
            GameControllerCore gameController = new()
            {
                backref = new TestDispatchUnityGameController(),
            };
            WorldObjectCore HQWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(0, 0),
            };

            BatteryComponentCore battery = new(100, 100);
            ResourcesComponentCore dispactherResources = new(
                new TestDispatchGameContent(),
                100,
                100
            );
            DispatchComponentCore dispatch = new(
                HQWorldObject,
                battery,
                dispactherResources,
                new TestDispatchGameContent(),
                DispatchComponentCore.Verbs.Retrieve.ToString(),
                "MiningDrill",
                "IronOre"
            );
            HQWorldObject.dispatchers = new List<DispatchComponentCore> { dispatch };

            WorldObjectCore targetWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(1, 1),
                worldObjectType = "IronOre",
            };

            WorldObjectCore receiverWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(2, 2),
            };

            ResourcesComponentCore receiverResources = new(new(), 100, 100);

            DispatchReceiverComponentCore receiver = new(
                receiverWorldObject,
                receiverResources,
                DispatchComponentCore.Verbs.Stockpile.ToString(),
                "MiningDrill"
            );
            receiverWorldObject.dispatchReceivers = new() { receiver };
            Assert.Null(receiver.dispatcher);

            gameController.worldObjects[new System.Numerics.Vector2(0, 0)] = new()
            {
                { "uuid-1", HQWorldObject },
            };
            gameController.worldObjects[new System.Numerics.Vector2(1, 1)] = new()
            {
                { "uuid-2", targetWorldObject },
            };
            gameController.worldObjects[new System.Numerics.Vector2(2, 2)] = new()
            {
                { "uuid-3", receiverWorldObject },
            };

            dispatch.Tick(gameController);
            Assert.NotNull(receiver.dispatcher);
        }

        [Fact]
        public void TestDoesNotAssignWhenNoResourcesAvailable()
        {
            GameControllerCore gameController = new()
            {
                backref = new TestDispatchUnityGameController(),
            };
            WorldObjectCore HQWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(0, 0),
            };

            BatteryComponentCore battery = new(100, 100);
            ResourcesComponentCore dispactherResources = new(
                new TestDispatchGameContent(),
                100,
                100
            );
            DispatchComponentCore dispatch = new(
                HQWorldObject,
                battery,
                dispactherResources,
                new TestDispatchGameContent(),
                DispatchComponentCore.Verbs.Retrieve.ToString(),
                "MiningDrill",
                DispatchComponentCore.Keywords.Me.ToString()
            );
            HQWorldObject.dispatchers = new List<DispatchComponentCore> { dispatch };

            WorldObjectCore targetWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(1, 1),
                worldObjectType = "IronOre",
            };

            gameController.worldObjects[new System.Numerics.Vector2(0, 0)] = new()
            {
                { "uuid-1", HQWorldObject },
            };
            gameController.worldObjects[new System.Numerics.Vector2(1, 1)] = new()
            {
                { "uuid-2", targetWorldObject },
            };

            List<Dictionary<uint, string>> alerts = dispatch.Tick(gameController);
            Assert.Equal(alerts.Count, 1);
            Assert.Equal(
                $"{dispatch.Description}: not enough available",
                alerts.First().Values.First()
            );
        }

        [Fact]
        public void TestDoesNotAssignWhenResourcesAlreadyPresent()
        {
            GameControllerCore gameController = new()
            {
                backref = new TestDispatchUnityGameController(),
            };
            WorldObjectCore HQWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(0, 0),
            };

            BatteryComponentCore battery = new(100, 100);
            ResourcesComponentCore dispactherResources = new(
                new TestDispatchGameContent(),
                100,
                100
            );
            dispactherResources.CreateResources("MiningDrill", 100);
            DispatchComponentCore dispatch = new(
                HQWorldObject,
                battery,
                dispactherResources,
                new TestDispatchGameContent(),
                DispatchComponentCore.Verbs.Deliver.ToString(),
                "MiningDrill",
                DispatchComponentCore.Keywords.Me.ToString()
            );
            HQWorldObject.dispatchers = new List<DispatchComponentCore> { dispatch };

            WorldObjectCore targetWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(1, 1),
                worldObjectType = "IronOre",
            };

            WorldObjectCore receiverWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(2, 2),
            };

            ResourcesComponentCore receiverResources = new(new(), 100, 100);

            DispatchReceiverComponentCore receiver = new(
                receiverWorldObject,
                receiverResources,
                "Deploy",
                "MiningDrill"
            );
            receiverWorldObject.dispatchReceivers = new() { receiver };
            Assert.Null(receiver.dispatcher);

            gameController.worldObjects[new System.Numerics.Vector2(0, 0)] = new()
            {
                { "uuid-1", HQWorldObject },
            };
            gameController.worldObjects[new System.Numerics.Vector2(1, 1)] = new()
            {
                { "uuid-2", targetWorldObject },
            };
            gameController.worldObjects[new System.Numerics.Vector2(2, 2)] = new()
            {
                { "uuid-3", receiverWorldObject },
            };

            List<Dictionary<uint, string>> alerts = dispatch.Tick(gameController);
            Assert.Null(receiver.dispatcher);
            Assert.Equal(alerts.Count, 1);
            Assert.Equal(
                $"{dispatch.Description}: no more required",
                alerts.First().Values.First()
            );
        }

        [Fact]
        public void TestDoesNotAssignTargetWhenVerbMismatch()
        {
            GameControllerCore gameController = new()
            {
                backref = new TestDispatchUnityGameController(),
            };
            WorldObjectCore HQWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(0, 0),
            };

            BatteryComponentCore battery = new(100, 100);
            ResourcesComponentCore dispactherResources = new(new(), 100, 100);
            DispatchComponentCore dispatch = new(
                HQWorldObject,
                battery,
                dispactherResources,
                new TestDispatchGameContent(),
                "Deploy",
                "MiningDrill",
                "IronOre"
            );
            HQWorldObject.dispatchers = new List<DispatchComponentCore> { dispatch };

            WorldObjectCore targetWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(1, 1),
            };

            WorldObjectCore receiverWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(2, 2),
                worldObjectType = "IronOre",
            };

            ResourcesComponentCore receiverResources = new(new(), 100, 100);

            DispatchReceiverComponentCore receiver = new(
                receiverWorldObject,
                receiverResources,
                "Retrieve",
                "MiningDrill"
            );
            receiverWorldObject.dispatchReceivers = new() { receiver };
            Assert.Null(receiver.dispatcher);

            gameController.worldObjects[new System.Numerics.Vector2(0, 0)] = new()
            {
                { "uuid-1", HQWorldObject },
            };
            gameController.worldObjects[new System.Numerics.Vector2(1, 1)] = new()
            {
                { "uuid-2", targetWorldObject },
            };
            gameController.worldObjects[new System.Numerics.Vector2(2, 2)] = new()
            {
                { "uuid-3", receiverWorldObject },
            };

            List<Dictionary<uint, string>> alerts = dispatch.Tick(gameController);
            Assert.Null(receiver.dispatcher);
            Assert.Equal(alerts.Count, 1);
            Assert.Equal(
                $"{dispatch.Description}: no receiver found",
                alerts.First().Values.First()
            );
        }

        [Fact]
        public void TestDoesNotAssignTargetWhenSubjectMismatch()
        {
            GameControllerCore gameController = new()
            {
                backref = new TestDispatchUnityGameController(),
            };
            WorldObjectCore HQWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(0, 0),
            };

            BatteryComponentCore battery = new(100, 100);
            ResourcesComponentCore dispactherResources = new(new(), 100, 100);
            DispatchComponentCore dispatch = new(
                HQWorldObject,
                battery,
                dispactherResources,
                new TestDispatchGameContent(),
                "Deploy",
                "MiningDrill",
                "IronOre"
            );
            HQWorldObject.dispatchers = new List<DispatchComponentCore> { dispatch };

            WorldObjectCore targetWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(1, 1),
                worldObjectType = "CopperOre",
            };

            WorldObjectCore receiverWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(2, 2),
            };

            ResourcesComponentCore receiverResources = new(new(), 100, 100);

            DispatchReceiverComponentCore receiver = new(
                receiverWorldObject,
                receiverResources,
                "Deploy",
                "Warehouse"
            );
            receiverWorldObject.dispatchReceivers = new() { receiver };
            Assert.Null(receiver.dispatcher);

            gameController.worldObjects[new System.Numerics.Vector2(0, 0)] = new()
            {
                { "uuid-1", HQWorldObject },
            };
            gameController.worldObjects[new System.Numerics.Vector2(1, 1)] = new()
            {
                { "uuid-2", targetWorldObject },
            };
            gameController.worldObjects[new System.Numerics.Vector2(2, 2)] = new()
            {
                { "uuid-3", receiverWorldObject },
            };

            List<Dictionary<uint, string>> alerts = dispatch.Tick(gameController);
            Assert.Null(receiver.dispatcher);
            Assert.Equal(alerts.Count, 1);
            Assert.Equal($"{dispatch.Description}: no target found", alerts.First().Values.First());
        }
    }
}
