using System;
using System.Collections.Generic;
using Assets.Scripts.Components.Core;
using Assets.Scripts.Core;
using Assets.Scripts.Unity;
using Assets.Scripts.WorldObjects.Unity;

namespace Assets.Scripts.WorldObjects.FactoryGame
{
    [Serializable]
    public class WorldObjectCoalPlant : WorldObject
    {
        public uint totalVolumeCapacity = 10000;
        public uint totalBatteryCapacity = 10000;
        public uint insertionRate = 5;
        public uint powerBurnRate = 5;
        public uint powerGainRate = 100;

        public override StatusDataComponentCore StatusData =>
            new()
            {
                Name = Util.HumanizedString(this.WorldObjectType),
                Energy = this.core.battery.PercentEnergyStatus,
                Resources = this.core.resources.ResourceInfo,
                Info = new()
                {
                    { "Dispatch", this.core.dispatch.Description },
                    { "Storage Volume", this.core.resources.UsedVolumeString },
                },
            };

        public override void Instantiate(SpawnQueueItem spawnQueueItem, GameContent gameContent)
        {
            base.Instantiate(spawnQueueItem, gameContent);
            this.core.resources = new(
                new FactoryGameContent(),
                weightCapacity: uint.MaxValue,
                volumeCapacity: this.totalVolumeCapacity
            );

            this.core.battery = new(capacity: this.totalBatteryCapacity);

            this.core.resourceInserters = new List<ResourceInserterComponentCore>()
            {
                new(
                    this.core.battery,
                    this.core.resources,
                    FactoryGameContent.Resources.Coal.ToString(),
                    this.insertionRate
                ),
                new(
                    this.core.battery,
                    this.core.resources,
                    FactoryGameContent.Resources.Coal.ToString(),
                    this.insertionRate
                ),
            };

            this.core.power = new PowerComponentCore(
                this.core.battery,
                this.core.resources,
                FactoryGameContent.Resources.Coal.ToString(),
                this.powerBurnRate,
                this.powerGainRate
            );

            this.core.dispatch = new(
                this.core,
                this.core.battery,
                // Deliver...
                DispatchComponentCore.Verbs.Deliver.ToString(),
                // ...coal...
                FactoryGameContent.Resources.Coal.ToString(),
                // ...to me.
                DispatchComponentCore.Keywords.Me.ToString()
            );
        }

        public override void Tick(GameController gameController)
        {
            base.Tick(gameController);
            foreach (ResourceInserterComponentCore inserter in this.core.resourceInserters)
            {
                inserter.Insert(this.core, gameController.core);
            }
            this.core.power.GeneratePower();
            this.core.battery.Balance(this.core, gameController.core);
        }
    }
}
