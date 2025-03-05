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

        public DispatchComponentCore(WorldObjectCore worldObject, BatteryComponentCore battery)
        {
            this.battery =
                battery
                ?? throw new GameControllerCore.MisconfigurationException(
                    "Dispatch component requires a battery component"
                );
            this.worldObject =
                worldObject
                ?? throw new GameControllerCore.MisconfigurationException(
                    "Dispatch component requires a world object"
                );
            if (this.worldObject.targetType == "")
            {
                throw new GameControllerCore.MisconfigurationException(
                    "Dispatch component requires a target type"
                );
            }
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

            // Acqiure a list of target locations
            System.Numerics.Vector2? targetLocation = gameController
                .worldObjects
                // For world objects that contain the target type
                .Where(worldObjects =>
                    worldObjects.Value.Any(worldObject =>
                        worldObject.Value.worldObjectType == this.worldObject.targetType
                    )
                )
                // For world objects that do not have a dispatch
                .Where(worldObjects =>
                    !this.dispatches.Keys.Contains(worldObjects.Value.First().Value.GridPosition)
                )
                // Order by distance to the current world object
                .OrderBy(worldObjects =>
                    System.Numerics.Vector2.Distance(
                        worldObjects.Value.First().Value.GridPosition,
                        this.worldObject.GridPosition
                    )
                )
                // Select the grid position of the target world objects
                .Select(worldObjects => worldObjects.Key)
                .FirstOrDefault();

            // Acquire a list of dispatch receivers awaiting a target
            DispatchReceiverComponentCore receiver = gameController
                .worldObjects
                // For all world objects
                .SelectMany(worldObjects => worldObjects.Value)
                // For all dispatch receivers
                .Select(worldObject => worldObject.Value.receiver)
                // Where the receiver is not null and is awaiting a target
                .Where(receiver => receiver != null)
                .Where(receiver => receiver.awaitingTarget)
                // Order by distance to the current world object
                .OrderBy(receiver =>
                    System.Numerics.Vector2.Distance(
                        receiver.worldObject.GridPosition,
                        this.worldObject.GridPosition
                    )
                )
                .FirstOrDefault();

            // Abort early if there is no target location or receiver
            if (targetLocation == null || receiver == null)
            {
                return;
            }

            // Assign the target to the receiver
            receiver.awaitingTarget = false;
            receiver.targetPosition = targetLocation.Value;
            receiver.dispatchHQ = this;
            this.dispatches[targetLocation.Value] = receiver;
        }
    }
}

namespace Assets.Scripts.Components.Tests
{
    using Assets.Scripts.Components.Core;
    using Xunit;

    public class DispatchComponentTest
    {
        [Fact]
        public void TestTrue()
        {
            GameControllerCore gameController = new();
            WorldObjectCore worldObject = new(null);
            BatteryComponentCore battery = new(100, 100);
            DispatchComponentCore dispatch = new(worldObject, battery);
            dispatch.Tick(gameController);
            Assert.True(true);
        }

        [Fact]
        public void TestAssignTarget()
        {
            GameControllerCore gameController = new();
            WorldObjectCore HQWorldObject = new(null)
            {
                targetType = "DINOSAURS",
                GridPosition = new System.Numerics.Vector2(0, 0),
            };

            BatteryComponentCore battery = new(100, 100);
            DispatchComponentCore dispatch = new(HQWorldObject, battery);
            HQWorldObject.dispatch = dispatch;

            WorldObjectCore targetWorldObject = new(null)
            {
                worldObjectType = "DINOSAURS",
                GridPosition = new System.Numerics.Vector2(1, 1),
            };

            WorldObjectCore receiverWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(2, 2),
            };

            DispatchReceiverComponentCore receiver = new(receiverWorldObject);
            receiverWorldObject.receiver = receiver;
            Assert.True(receiver.awaitingTarget);

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
            Assert.False(receiver.awaitingTarget);
        }
    }
}
