using System.Collections.Generic;
using System.Linq;
using Assets.Scripts.Components.Core;
using Assets.Scripts.Core;
using Assets.Scripts.Unity;

namespace Assets.Scripts.WorldObjects.FactoryGame
{
    public class WorldObjectTruck : WorldObject
    {
        public uint insertionRate = 1;
        public uint totalVolumeCapacity = 500;
        public uint totalWeightCapacity = 500;
        public uint totalBatteryCapacity = 250;

        public override StatusDataComponentCore StatusData =>
            new()
            {
                Name = this.WorldObjectType,
                Energy = this.core.battery.PercentEnergyStatus,
                Receivers = this
                    .core.dispatchReceivers.Select(receiver => receiver.Description)
                    .ToList(),
                Resources = this.core.resources.ResourceInfo,
                Info = new()
                {
                    { "Storage Volume", this.core.resources.UsedVolumeString },
                    { "Target", this.core.dispatchReceivers.First().Description },
                },
            };

        public override void Instantiate(SpawnQueueItem spawnQueueItem, GameContent gameContent)
        {
            base.Instantiate(spawnQueueItem, gameContent);
            this.core.mobile = true;
            this.core.resources = new(
                new FactoryGameContent(),
                weightCapacity: this.totalWeightCapacity,
                volumeCapacity: this.totalVolumeCapacity
            );
            this.core.battery = new(capacity: this.totalBatteryCapacity);
            // Mobile objects can only ever have 1 dispatch receiver
            this.core.dispatchReceivers = new List<DispatchReceiverComponentCore>
            {
                new(
                    this.core,
                    this.core.resources,
                    DispatchComponentCore.Verbs.Retrieve.ToString(),
                    FactoryGameContent.Spawnables.MiningDrill.ToString()
                ),
            };
            this.core.movement = new MovementComponentCore(this);
            this.core.resourceRetriever = new ResourceRetrieverCore(
                this.core,
                this.core.resources,
                this.core.battery,
                this.core.dispatchReceivers.First(),
                new FactoryGameContent(),
                FactoryGameContent.Spawnables.MiningDrill.ToString(),
                1
            );
            this.core.resourceInserters = new List<ResourceInserterComponentCore>
            {
                new(
                    this.core.battery,
                    this.core.resources,
                    FactoryGameContent.Spawnables.MiningDrill.ToString(), // Mining drill
                    this.insertionRate
                ),
            };
        }

        public override void Tick(GameController gameController)
        {
            base.Tick(gameController);
            this.core.movement.Tick(gameController);
            foreach (DispatchReceiverComponentCore receiver in this.core.dispatchReceivers)
            {
                receiver.Tick();
            }
            this.core.resourceRetriever.Tick();
        }
    }
}
