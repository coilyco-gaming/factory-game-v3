namespace Assets.Scripts.Components.Core
{
    using System.Collections.Generic;
    using System.Linq;
    using Assets.Scripts.Core;

    public class DispatchComponentCore
    {
        // Example descriptions:
        //  - Retrieve power lines from me
        //  - Deploy mining drill to aluminum ore
        //  - Collect coal to me
        private string DescriptionToOrFrom =>
            this.receiverVerb == DispatchComponentCore.Verbs.Retrieve.ToString()
            || this.receiverVerb == DispatchComponentCore.Verbs.Collect.ToString()
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

        public enum Verbs
        {
            // Collect <=> Deliver
            Collect,
            Deliver,

            // Retrieve <=> Deploy
            Retrieve,
            Deploy,
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

            // Acqiure list of target locations
            //  - If the receiver is me, then return the current world object location
            //  - If the receiver is not me, then return the list of world objects
            //    whose world object type match the receiver object
            //
            // Examples:
            //  - Deploy mining drill to aluminum ore
            //    targetLocations = < aluminum ore grid positions >
            //  - Deliver aluminum ore to me, Collect aluminum ore from me
            //    targetLocations = < current world object grid position >
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

            // Don't dispatch to target if there's already another dispatch
            // of the same type assigned to the same location
            List<DispatchComponentCore> dispatchers = gameController
                .worldObjects
                // For all world objects
                .SelectMany(worldObjects => worldObjects.Value)
                .Where(worldObject =>
                    // Where the world object has dispatchers
                    worldObject.Value.dispatchers != null
                )
                // For all dispatchers
                .SelectMany(worldObject => worldObject.Value.dispatchers)
                .Where(dispatcher =>
                    // Where the dispatcher is not null and is awaiting a target
                    dispatcher != null
                    && dispatcher.receiver != null
                    // Where the dispatcher Subject and verb match the dispatch
                    && dispatcher.receiverSubject == this.receiverSubject
                    && this.receiverVerb == dispatcher.receiverVerb
                )
                .ToList();

            // Remove targets target already at the target locations
            // represents by the existing dispatchers
            targetLocations = targetLocations
                .Where(targetLocation =>
                    !dispatchers.Any(dispatcher =>
                        dispatcher.receiver != null
                        && dispatcher.receiver.targetPosition == targetLocation
                    )
                )
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
                    && this.receiverVerb == receiver.receiverVerb
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
                    "aluminumBars",
                    new Item(
                        "aluminumBars",
                        stackSize: 10,
                        ingredients: new Dictionary<string, uint> { { "aluminumOre", 5 } }
                    )
                },
                {
                    "MiningDrill",
                    new Item(
                        "MiningDrill",
                        stackSize: 1,
                        craftTime: 3,
                        ingredients: new Dictionary<string, uint> { { "aluminumBars", 5 } }
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
                "aluminumOre"
            );
            HQWorldObject.dispatchers = new List<DispatchComponentCore> { dispatch };

            WorldObjectCore targetWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(1, 1),
                worldObjectType = "aluminumOre",
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
        public void TestNoDuplicateDispatches()
        {
            GameControllerCore gameController = new()
            {
                backref = new TestDispatchUnityGameController(),
            };

            // HQ 1
            WorldObjectCore HQWorldObject1 = new(null)
            {
                GridPosition = new System.Numerics.Vector2(0, 0),
            };
            BatteryComponentCore battery1 = new(100, 100);
            ResourcesComponentCore dispactherResources1 = new(
                new TestDispatchGameContent(),
                100,
                100
            );
            DispatchComponentCore dispatch1 = new(
                HQWorldObject1,
                battery1,
                dispactherResources1,
                new TestDispatchGameContent(),
                "Deploy",
                "MiningDrill",
                "aluminumOre"
            );
            HQWorldObject1.dispatchers = new List<DispatchComponentCore> { dispatch1 };

            // HQ 2
            WorldObjectCore HQWorldObject2 = new(null)
            {
                GridPosition = new System.Numerics.Vector2(0, 0),
            };
            BatteryComponentCore battery2 = new(100, 100);
            ResourcesComponentCore dispactherResources2 = new(
                new TestDispatchGameContent(),
                100,
                100
            );
            DispatchComponentCore dispatch2 = new(
                HQWorldObject2,
                battery2,
                dispactherResources2,
                new TestDispatchGameContent(),
                "Deploy",
                "MiningDrill",
                "aluminumOre"
            );
            HQWorldObject2.dispatchers = new List<DispatchComponentCore> { dispatch2 };

            // target
            WorldObjectCore targetWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(1, 1),
                worldObjectType = "aluminumOre",
            };

            // receiver 1
            WorldObjectCore receiverWorldObject1 = new(null)
            {
                GridPosition = new System.Numerics.Vector2(2, 2),
            };
            ResourcesComponentCore receiverResources1 = new(new(), 100, 100);
            DispatchReceiverComponentCore receiver1 = new(
                receiverWorldObject1,
                receiverResources1,
                "Deploy",
                "MiningDrill"
            );
            receiverWorldObject1.dispatchReceivers = new() { receiver1 };

            // receiver 2
            WorldObjectCore receiverWorldObject2 = new(null)
            {
                GridPosition = new System.Numerics.Vector2(2, 2),
            };
            ResourcesComponentCore receiverResources2 = new(new(), 100, 100);
            DispatchReceiverComponentCore receiver2 = new(
                receiverWorldObject2,
                receiverResources2,
                "Deploy",
                "MiningDrill"
            );
            receiverWorldObject2.dispatchReceivers = new() { receiver2 };

            gameController.worldObjects[new System.Numerics.Vector2(-1, -1)] = new()
            {
                { "uuid-0", HQWorldObject1 },
            };
            gameController.worldObjects[new System.Numerics.Vector2(0, 0)] = new()
            {
                { "uuid-1", HQWorldObject2 },
            };
            gameController.worldObjects[new System.Numerics.Vector2(1, 1)] = new()
            {
                { "uuid-2", targetWorldObject },
            };
            gameController.worldObjects[new System.Numerics.Vector2(2, 2)] = new()
            {
                { "uuid-3", receiverWorldObject1 },
            };
            gameController.worldObjects[new System.Numerics.Vector2(3, 3)] = new()
            {
                { "uuid-4", receiverWorldObject2 },
            };

            dispatch1.Tick(gameController);
            dispatch2.Tick(gameController);
            Assert.NotNull(receiver1.dispatcher);
            Assert.Null(receiver2.dispatcher);
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
                worldObjectType = "aluminumOre",
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

        // [Fact]
        // public void TestDoesNotAssignWhenResourcesAlreadyPresent()
        // {
        //     GameControllerCore gameController = new()
        //     {
        //         backref = new TestDispatchUnityGameController(),
        //     };
        //     WorldObjectCore HQWorldObject = new(null)
        //     {
        //         GridPosition = new System.Numerics.Vector2(0, 0),
        //     };

        //     BatteryComponentCore battery = new(100, 100);
        //     ResourcesComponentCore dispactherResources = new(
        //         new TestDispatchGameContent(),
        //         100,
        //         100
        //     );
        //     dispactherResources.CreateResources("MiningDrill", 100);
        //     DispatchComponentCore dispatch = new(
        //         HQWorldObject,
        //         battery,
        //         dispactherResources,
        //         new TestDispatchGameContent(),
        //         DispatchComponentCore.Verbs.Collect.ToString(),
        //         "MiningDrill",
        //         DispatchComponentCore.Keywords.Me.ToString()
        //     );
        //     HQWorldObject.dispatchers = new List<DispatchComponentCore> { dispatch };

        //     WorldObjectCore targetWorldObject = new(null)
        //     {
        //         GridPosition = new System.Numerics.Vector2(1, 1),
        //         worldObjectType = "aluminumOre",
        //     };

        //     WorldObjectCore receiverWorldObject = new(null)
        //     {
        //         GridPosition = new System.Numerics.Vector2(2, 2),
        //     };

        //     ResourcesComponentCore receiverResources = new(new(), 100, 100);

        //     DispatchReceiverComponentCore receiver = new(
        //         receiverWorldObject,
        //         receiverResources,
        //         "Deploy",
        //         "MiningDrill"
        //     );
        //     receiverWorldObject.dispatchReceivers = new() { receiver };
        //     Assert.Null(receiver.dispatcher);

        //     gameController.worldObjects[new System.Numerics.Vector2(0, 0)] = new()
        //     {
        //         { "uuid-1", HQWorldObject },
        //     };
        //     gameController.worldObjects[new System.Numerics.Vector2(1, 1)] = new()
        //     {
        //         { "uuid-2", targetWorldObject },
        //     };
        //     gameController.worldObjects[new System.Numerics.Vector2(2, 2)] = new()
        //     {
        //         { "uuid-3", receiverWorldObject },
        //     };

        //     List<Dictionary<uint, string>> alerts = dispatch.Tick(gameController);
        //     Assert.Null(receiver.dispatcher);
        //     Assert.Equal(alerts.Count, 1);
        //     Assert.Equal(
        //         $"{dispatch.Description}: no more required",
        //         alerts.First().Values.First()
        //     );
        // }

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
                "aluminumOre"
            );
            HQWorldObject.dispatchers = new List<DispatchComponentCore> { dispatch };

            WorldObjectCore targetWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(1, 1),
            };

            WorldObjectCore receiverWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(2, 2),
                worldObjectType = "aluminumOre",
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
                "aluminumOre"
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
