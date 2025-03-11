using System.Collections.Generic;
using System.Linq;
using Assets.Scripts.Core;
using Assets.Scripts.WorldObjects.Core;

namespace Assets.Scripts.Components.Core
{
    public class DispatchComponentCore
    {
        // Example descriptions:
        //  - Retrieve power lines from me
        //  - Deploy mining drill to iron ore
        //  - Deliver coal to me
        private string DescriptionToOrFrom =>
            this.receiverVerb == DispatchComponentCore.Verbs.Retrieve.ToString() ? "from" : "to";
        private string DescriptionSubject => Util.HumanizedString(this.receiverSubject);
        private string DescriptionObject => Util.HumanizedString(this.receiverObject);
        public string Description =>
            $"{this.receiverVerb} {this.DescriptionSubject} {this.DescriptionToOrFrom} {this.DescriptionObject}".ToLower();

        public WorldObjectCore worldObject;
        private BatteryComponentCore battery;
        private ResourcesComponentCore resources;
        private GameContent gameContent;
        private string receiverVerb = "VERB";
        private string receiverSubject = "SUBJECT";
        private string receiverObject = "OBJECT";

        public enum Verbs
        {
            Deploy,
            Deliver,
            Retrieve,
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

        public void Tick(GameControllerCore gameController)
        {
            // If the dispatch subject is an item,
            // then skip delivery dispatch if you have more than a stack of the item
            if (
                this.gameContent != null
                && this.gameContent.Items != null
                && this.gameContent.Items.ContainsKey(this.receiverObject)
                && this.resources.resources != null
                && this.resources.resources.ContainsKey(this.receiverObject)
                && this.receiverVerb == DispatchComponentCore.Verbs.Deliver.ToString()
                && this.resources.resources[this.receiverObject]
                    >= this.gameContent.Items[this.receiverObject].StackSize
            )
            {
                return;
            }

            // If the dispatch subject is an item,
            // then skip retrieve dispatch if you have less than a stack of the item
            if (
                this.gameContent != null
                && this.gameContent.Items != null
                && this.gameContent.Items.ContainsKey(this.receiverObject)
                && this.resources.resources != null
                && this.resources.resources.ContainsKey(this.receiverObject)
                && this.receiverVerb == DispatchComponentCore.Verbs.Retrieve.ToString()
                && this.resources.resources[this.receiverObject]
                    < this.gameContent.Items[this.receiverObject].StackSize
            )
            {
                return;
            }

            // Abort early if the battery is empty
            try
            {
                this.battery.Energy -= 1;
            }
            catch (BatteryComponentCore.BatteryCapacityException)
            {
                return;
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

            if (targetLocations.Count == 0)
            {
                return;
            }

            // Get the first receiver awaiting a target
            DispatchReceiverComponentCore receiver = gameController
                .worldObjects
                // For all world objects
                .SelectMany(worldObjects => worldObjects.Value)
                // For all dispatch receivers
                .Select(worldObject => worldObject.Value.dispatchReceiver)
                .Where(receiver =>
                    // Where the receiver is not null and is awaiting a target
                    receiver != null
                    && receiver.dispatcher == null
                    // Where the receiver Subject and verb match the dispatch
                    && receiver.receiverSubject == this.receiverSubject
                    && receiver.receiverVerb == this.receiverVerb
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
                return;
            }

            // Assign the target to the receiver
            receiver.targetPosition = targetLocations[0];
            receiver.dispatcher = this;
        }
    }
}

namespace Assets.Scripts.Components.Tests
{
    using Assets.Scripts.Components.Core;
    using Xunit;
    using Xunit.Abstractions;

    internal class TestDispatchGameContent : GameContent
    {
        public override Dictionary<string, Item> Items { get; } =
            new()
            {
                { "IRON_BARS", new Item("IRON_BARS", stackSize: 100) },
                {
                    "MINING_DRILL",
                    new Item(
                        "MINING_DRILL",
                        stackSize: 10,
                        craftTime: 3,
                        ingredients: new Dictionary<string, uint> { { "IRON_BARS", 5 } }
                    )
                },
            };
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
            GameControllerCore gameController = new();
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
            GameControllerCore gameController = new();
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
                "DEPLOY",
                "MINING_DRILL",
                "IRON_ORE"
            );
            HQWorldObject.dispatchers = new List<DispatchComponentCore> { dispatch };

            WorldObjectCore targetWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(1, 1),
                worldObjectType = "IRON_ORE",
            };

            WorldObjectCore receiverWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(2, 2),
            };

            ResourcesComponentCore receiverResources = new(new(), 100, 100);

            DispatchReceiverComponentCore receiver = new(
                receiverWorldObject,
                receiverResources,
                "DEPLOY",
                "MINING_DRILL"
            );
            receiverWorldObject.dispatchReceiver = receiver;
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
        public void TestDoesNotAssignWhenResourcesAlreadyPresent()
        {
            GameControllerCore gameController = new();
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
            dispactherResources.CreateResources("MINING_DRILL", 1);
            DispatchComponentCore dispatch = new(
                HQWorldObject,
                battery,
                dispactherResources,
                new TestDispatchGameContent(),
                "DEPLOY",
                "MINING_DRILL",
                "IRON_ORE"
            );
            HQWorldObject.dispatchers = new List<DispatchComponentCore> { dispatch };

            WorldObjectCore targetWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(1, 1),
                worldObjectType = "IRON_ORE",
            };

            WorldObjectCore receiverWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(2, 2),
            };

            ResourcesComponentCore receiverResources = new(new(), 100, 100);

            DispatchReceiverComponentCore receiver = new(
                receiverWorldObject,
                receiverResources,
                "DEPLOY",
                "MINING_DRILL"
            );
            receiverWorldObject.dispatchReceiver = receiver;
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
        public void TestDoesNotAssignTargetWhenObjectMismatch()
        {
            GameControllerCore gameController = new();
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
                "DEPLOY",
                "MINING_DRILL",
                "IRON_ORE"
            );
            HQWorldObject.dispatchers = new List<DispatchComponentCore> { dispatch };

            WorldObjectCore targetWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(1, 1),
                worldObjectType = "COPPER_ORE",
            };

            WorldObjectCore receiverWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(2, 2),
            };

            ResourcesComponentCore receiverResources = new(new(), 100, 100);

            DispatchReceiverComponentCore receiver = new(
                receiverWorldObject,
                receiverResources,
                "DEPLOY",
                "MINING_DRILL"
            );
            receiverWorldObject.dispatchReceiver = receiver;
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
            Assert.Null(receiver.dispatcher);
        }

        [Fact]
        public void TestDoesNotAssignTargetWhenVerbMismatch()
        {
            GameControllerCore gameController = new();
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
                "DEPLOY",
                "MINING_DRILL",
                "IRON_ORE"
            );
            HQWorldObject.dispatchers = new List<DispatchComponentCore> { dispatch };

            WorldObjectCore targetWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(1, 1),
            };

            WorldObjectCore receiverWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(2, 2),
            };

            ResourcesComponentCore receiverResources = new(new(), 100, 100);

            DispatchReceiverComponentCore receiver = new(
                receiverWorldObject,
                receiverResources,
                "RETRIEVE",
                "MINING_DRILL"
            );
            receiverWorldObject.dispatchReceiver = receiver;
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
            Assert.Null(receiver.dispatcher);
        }

        [Fact]
        public void TestDoesNotAssignTargetWhenSubjectMismatch()
        {
            GameControllerCore gameController = new();
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
                "DEPLOY",
                "MINING_DRILL",
                "IRON_ORE"
            );
            HQWorldObject.dispatchers = new List<DispatchComponentCore> { dispatch };

            WorldObjectCore targetWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(1, 1),
            };

            WorldObjectCore receiverWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(2, 2),
            };

            ResourcesComponentCore receiverResources = new(new(), 100, 100);

            DispatchReceiverComponentCore receiver = new(
                receiverWorldObject,
                receiverResources,
                "DEPLOY",
                "WAREHOUSE"
            );
            receiverWorldObject.dispatchReceiver = receiver;
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
            Assert.Null(receiver.dispatcher);
        }
    }
}
