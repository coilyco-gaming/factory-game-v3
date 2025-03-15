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
                Resources = this.core.resources.ResourceInfo,
                Info = new() { { "Storage Volume", this.core.resources.UsedVolumeString } },
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
                new(this.core, this.core.resources, this.core.targetType, this.core.targetSubType),
            };
            this.core.deployments = new List<DeploymentComponentCore>();
            foreach (DispatchReceiverComponentCore dispatchReceiver in this.core.dispatchReceivers)
            {
                this.core.deployments.Add(new(this.core.resources, dispatchReceiver));
            }
            this.core.movement = new MovementComponentCore(this);
            this.core.resourceRetriever = new ResourceRetrieverCore(
                this.core,
                this.core.resources,
                this.core.battery,
                this.core.dispatchReceivers.First(),
                gameContent,
                this.core.targetSubType,
                gameContent.Items[this.core.targetSubType].StackSize
            );
        }

        public override void Tick(GameController gameController)
        {
            base.Tick(gameController);
            this.core.movement.Tick(gameController);
            foreach (DispatchReceiverComponentCore receiver in this.core.dispatchReceivers)
            {
                receiver.Tick();
            }
            foreach (DeploymentComponentCore deployment in this.core.deployments)
            {
                deployment.Tick(gameController.core);
            }
            this.core.resourceRetriever.Tick();
        }
    }
}
