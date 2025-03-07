using System.Collections.Generic;
using System.Linq;
using Assets.Scripts.Core;
using Assets.Scripts.WorldObjects.Core;

namespace Assets.Scripts.Components.Core
{
    public class DispatchComponentCore
    {
        public int ActiveDispatches => this.dispatches.Count;
        public Dictionary<System.Numerics.Vector2, DispatchReceiverComponentCore> dispatches =
            new();
        private WorldObjectCore worldObject;
        private BatteryComponentCore battery;
        private string receiverVerb;
        private string receiverNoun;
        private string receiverObject;

        public enum Keywords
        {
            TargetSelf,
        }

        public DispatchComponentCore(
            WorldObjectCore worldObject,
            BatteryComponentCore battery,
            string receiverVerb,
            string receiverNoun,
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
            this.receiverVerb = receiverVerb;
            this.receiverNoun = receiverNoun;
            this.receiverObject = receiverObject;
        }

        public void Tick(GameControllerCore gameController)
        {
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
                this.receiverObject == DispatchComponentCore.Keywords.TargetSelf.ToString()
                    ? new List<System.Numerics.Vector2> { this.worldObject.gridPosition }
                    : gameController
                        .worldObjects
                        // For world object locations that do not already have a dispatch
                        .Where(worldObjects => !this.dispatches.Keys.Contains(worldObjects.Key))
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
                .Select(worldObject => worldObject.Value.receiver)
                .Where(receiver =>
                    // Where the receiver is not null and is awaiting a target
                    receiver != null
                    && receiver.dispatcher == null
                    // Where the receiver noun and verb match the dispatch
                    && receiver.receiverNoun == this.receiverNoun
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
            this.dispatches[targetLocations[0]] = receiver;
        }
    }
}

namespace Assets.Scripts.Components.Tests
{
    using Assets.Scripts.Components.Core;
    using Xunit;
    using Xunit.Abstractions;

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
            DispatchComponentCore dispatch = new(worldObject, battery, "", "", "");
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
            DispatchComponentCore dispatch = new(
                HQWorldObject,
                battery,
                "DEPLOY",
                "MINING_DRILL",
                "IRON_ORE"
            );
            HQWorldObject.dispatch = dispatch;

            WorldObjectCore targetWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(1, 1),
                worldObjectType = "IRON_ORE",
            };

            WorldObjectCore receiverWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(2, 2),
            };

            DispatchReceiverComponentCore receiver = new(
                receiverWorldObject,
                "DEPLOY",
                "MINING_DRILL"
            );
            receiverWorldObject.receiver = receiver;
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
            DispatchComponentCore dispatch = new(
                HQWorldObject,
                battery,
                "DEPLOY",
                "MINING_DRILL",
                "IRON_ORE"
            );
            HQWorldObject.dispatch = dispatch;

            WorldObjectCore targetWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(1, 1),
                worldObjectType = "COPPER_ORE",
            };

            WorldObjectCore receiverWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(2, 2),
            };

            DispatchReceiverComponentCore receiver = new(
                receiverWorldObject,
                "DEPLOY",
                "MINING_DRILL"
            );
            receiverWorldObject.receiver = receiver;
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
            DispatchComponentCore dispatch = new(
                HQWorldObject,
                battery,
                "DEPLOY",
                "MINING_DRILL",
                "IRON_ORE"
            );
            HQWorldObject.dispatch = dispatch;

            WorldObjectCore targetWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(1, 1),
            };

            WorldObjectCore receiverWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(2, 2),
            };

            DispatchReceiverComponentCore receiver = new(
                receiverWorldObject,
                "RETRIEVE",
                "MINING_DRILL"
            );
            receiverWorldObject.receiver = receiver;
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
        public void TestDoesNotAssignTargetWhenNounMismatch()
        {
            GameControllerCore gameController = new();
            WorldObjectCore HQWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(0, 0),
            };

            BatteryComponentCore battery = new(100, 100);
            DispatchComponentCore dispatch = new(
                HQWorldObject,
                battery,
                "DEPLOY",
                "MINING_DRILL",
                "IRON_ORE"
            );
            HQWorldObject.dispatch = dispatch;

            WorldObjectCore targetWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(1, 1),
            };

            WorldObjectCore receiverWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(2, 2),
            };

            DispatchReceiverComponentCore receiver = new(
                receiverWorldObject,
                "DEPLOY",
                "WAREHOUSE"
            );
            receiverWorldObject.receiver = receiver;
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
