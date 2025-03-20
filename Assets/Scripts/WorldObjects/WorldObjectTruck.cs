using System.Collections.Generic;
using System.Linq;
using Assets.Scripts.Components.Core;
using Assets.Scripts.Core;
using Assets.Scripts.Unity;

namespace Assets.Scripts.WorldObjects.FactoryGame
{
    public class WorldObjectTruck : WorldObject
    {
        private uint totalVolumeCapacity = 500;
        private uint totalWeightCapacity = 500;
        private uint totalBatteryCapacity = 250;

        public override StatusDataComponentCore StatusData =>
            new()
            {
                Name = this.WorldObjectType,
                Energy = this.core.battery.PercentEnergyStatus,
                Receivers = this
                    .core.dispatchReceivers.Select(receiver => receiver.Description)
                    .ToList(),
                DispatchHistory = this
                    .core.dispatchReceivers.First()
                    .dispatchHistory.Select(kvp =>
                        $"{kvp.Key.Description} via {kvp.Key.worldObject.gridPosition} at tick {kvp.Value.Item1} for target {kvp.Value.Item2}"
                    )
                    .ToList(),
                Resources = this.core.resources.ResourceInfo,
                Info = new() { { "Storage Volume", this.core.resources.UsedVolumeString } },
                Alerts = this.core.alerts.Count == 0 ? null : this.core.alerts,
            };

        public override void Instantiate(SpawnQueueItem spawnQueueItem, GameContent gameContent)
        {
            base.Instantiate(spawnQueueItem, gameContent);
            this.core.mobile = true;
            this.core.resources = new(
                gameContent,
                weightCapacity: this.totalWeightCapacity,
                volumeCapacity: this.totalVolumeCapacity
            )
            {
                resources = spawnQueueItem.resources,
            };
            this.core.battery = new(capacity: this.totalBatteryCapacity);
            // Mobile objects can only ever have 1 dispatch receiver
            this.core.dispatchReceivers = new List<DispatchReceiverComponentCore>
            {
                new(this.core, this.core.targetType, this.core.targetSubType),
            };
            this.core.deployments = new List<DeploymentComponentCore>();
            foreach (DispatchReceiverComponentCore dispatchReceiver in this.core.dispatchReceivers)
            {
                this.core.deployments.Add(new());
            }
            this.core.movement = new MovementComponentCore();
            this.core.resourceRetriever = new ResourceRetrieverCore(
                gameContent,
                this.core.targetSubType,
                gameContent.Items[this.core.targetSubType].StackSize
            );
        }

        public override void Tick(GameController gameController)
        {
            base.Tick(gameController);
            this.core.CreateAlert(
                gameController.core,
                this.core.movement.Tick(gameController.core, this.core)
            );
            foreach (DispatchReceiverComponentCore receiver in this.core.dispatchReceivers)
            {
                this.core.CreateAlert(
                    gameController.core,
                    receiver.Tick(gameController.core, this.core)
                );
            }
            foreach (DeploymentComponentCore deployment in this.core.deployments)
            {
                this.core.CreateAlert(
                    gameController.core,
                    deployment.Tick(gameController.core, this.core)
                );
            }
            this.core.CreateAlert(
                gameController.core,
                this.core.resourceRetriever.Tick(gameController.core, this.core)
            );
        }
    }
}
