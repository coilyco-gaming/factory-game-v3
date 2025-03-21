using System;
using System.Collections.Generic;
using System.Linq;
using Assets.Scripts.Components.Core;
using Assets.Scripts.Core;
using Assets.Scripts.Unity;

namespace Assets.Scripts.WorldObjects.FactoryGame
{
    [Serializable]
    public class WorldObjectCoalPlant : WorldObject
    {
        private uint totalVolumeCapacity = 10000;
        private uint totalBatteryCapacity = 10000;
        private uint powerBurnRate = 4;
        private uint powerGainRate = 160;
        private uint insertionRate = 20;
        private int miningSpeed = 20;
        private int miningEnergyCost = 2;

        public override StatusDataComponentCore StatusData =>
            new()
            {
                Name = Util.HumanizedString(this.WorldObjectType),
                Energy = this.core.battery.PercentEnergyStatus,
                Dispatchers = this
                    .core.dispatchers.Select(dispatcher => dispatcher.Description)
                    .ToList(),
                Resources = this.core.resources.ResourceInfo,
                Info = new() { { "Storage Volume", this.core.resources.UsedVolumeString } },
                Alerts = this.core.alerts.Count == 0 ? null : this.core.alerts,
            };

        public override void Instantiate(SpawnQueueItem spawnQueueItem, GameContent gameContent)
        {
            base.Instantiate(spawnQueueItem, gameContent);
            this.core.resources = new(
                gameContent,
                weightCapacity: uint.MaxValue,
                volumeCapacity: this.totalVolumeCapacity
            )
            {
                resources = spawnQueueItem.resources,
            };

            this.core.battery = new(capacity: this.totalBatteryCapacity);

            this.core.power = new PowerComponentCore(
                this.core.targetType,
                this.powerBurnRate,
                this.powerGainRate
            );

            this.core.resourceInserters = new() { new(this.core.targetType, this.insertionRate) };

            this.core.dispatchers = new List<DispatchComponentCore>()
            {
                new(
                    gameContent,
                    // Deliver...
                    DispatchComponentCore.Verbs.Deliver.ToString(),
                    // ...coal...
                    this.core.targetType,
                    // ...to me.
                    DispatchComponentCore.Keywords.Me.ToString()
                ),
            };
            this.core.mining = new MiningComponentCore(
                gameContent,
                FactoryGameContent.Resources.Coal.ToString(),
                this.miningSpeed,
                this.miningEnergyCost
            );
            this.core.powerLine = new PowerLineComponentCore(
                FactoryGameContent.Spawnables.PowerLines.ToString()
            );
        }

        public override void Tick(GameController gameController)
        {
            base.Tick(gameController);
            foreach (DispatchComponentCore dispatcher in this.core.dispatchers)
            {
                this.core.CreateAlert(
                    gameController.core,
                    dispatcher.Tick(gameController.core, this.core)
                );
            }
            foreach (ResourceInserterComponentCore inserter in this.core.resourceInserters)
            {
                this.core.CreateAlert(
                    gameController.core,
                    inserter.Tick(gameController.core, this.core)
                );
            }
            this.core.CreateAlert(
                gameController.core,
                this.core.mining.Tick(gameController.core, this.core)
            );
            this.core.CreateAlert(
                gameController.core,
                this.core.power.Tick(gameController.core, this.core)
            );
            this.core.CreateAlert(
                gameController.core,
                this.core.battery.Tick(gameController.core, this.core)
            );
            this.core.CreateAlert(
                gameController.core,
                this.core.powerLine.Tick(gameController.core, this.core)
            );
        }
    }
}
