using System.Collections.Generic;
using Assets.Scripts.Components.Core;
using Assets.Scripts.Core;
using Assets.Scripts.Unity;
using Assets.Scripts.WorldObjects.Unity;

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
                Resources = this.core.resources.ResourceInfo,
                Info = new()
                {
                    { "Storage Volume", this.core.resources.UsedVolumeString },
                    {
                        "Target Description",
                        this.core.dispatchReceiver != null
                        && this.core.dispatchReceiver.dispatcher != null
                            ? this.core.dispatchReceiver.dispatcher.Description
                            : "awaiting target"
                    },
                    {
                        "Target Location",
                        this.core.dispatchReceiver != null
                        && this.core.dispatchReceiver.dispatcher != null
                            ? this.core.dispatchReceiver.targetPosition.ToString()
                            : "awaiting target"
                    },
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
            this.core.dispatchReceiver = new(
                this.core,
                this.core.resources,
                DispatchComponentCore.Verbs.Retrieve.ToString(), // Deploy
                FactoryGameContent.Spawnables.MiningDrill.ToString() // Mining drill
            ); // TODO: rotate around possible choices
            this.core.movement = new MovementComponentCore(this, this.core.dispatchReceiver);
            this.core.resourceRetriever = new ResourceRetrieverCore(
                this.core,
                this.core.resources,
                this.core.battery,
                this.core.dispatchReceiver,
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
            this.core.resourceRetriever.Tick();
        }
    }
}
