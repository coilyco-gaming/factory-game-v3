using System;
using Assets.Scripts.Components.Core;
using Assets.Scripts.ScriptableObject;
using Assets.Scripts.Unity;
using Assets.Scripts.WorldObjects.Unity;

namespace Assets.Scripts.WorldObjects.FactoryGame
{
    public class WorldObjectTruck : WorldObject
    {
        public uint totalVolumeCapacity = 500;
        public uint totalWeightCapacity = 500;
        public uint totalBatteryCapacity = 250;

        public override void Instantiate(SpawnQueueItem spawnQueueItem)
        {
            base.Instantiate(spawnQueueItem);
            this.core.resources = new(
                new FactoryGameContent(),
                weightCapacity: this.totalWeightCapacity,
                volumeCapacity: this.totalVolumeCapacity
            );
            this.core.battery = new(capacity: this.totalBatteryCapacity);
            this.core.receiver = new(
                this.core,
                DispatchComponentCore.Verbs.Retrieve.ToString(), // Deploy
                FactoryGameContent.Spawnables.MiningDrill.ToString() // Mining drill
            ); // TODO: rotate around possible choices
            this.core.movement = new MovementComponentCore(this, this.core.receiver);
        }

        public override void Tick(GameController gameController)
        {
            base.Tick(gameController);
            this.core.movement.Tick(gameController);
        }

        protected override Func<StatusDataComponentCore.StatusData> GetStatusData()
        {
            return () =>
                new()
                {
                    Name = this.WorldObjectType,
                    Resources = this.core.resources.ResourceInfo,
                    Info = new()
                    {
                        { "Storage Volume", this.core.resources.UsedVolumeString },
                        { "Energy", this.core.battery.PercentEnergyStatus },
                        {
                            "Target Description",
                            this.core.receiver != null && this.core.receiver.dispatcher != null
                                ? this.core.receiver.dispatcher.ReceiverDescription
                                : "awaiting target"
                        },
                        {
                            "Target Location",
                            this.core.receiver != null && this.core.receiver.dispatcher != null
                                ? this.core.receiver.targetPosition.ToString()
                                : "awaiting target"
                        },
                    },
                };
        }
    }
}
