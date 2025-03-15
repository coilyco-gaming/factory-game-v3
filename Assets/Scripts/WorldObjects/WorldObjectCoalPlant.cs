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
                Dispatchers = this
                    .core.dispatchers.Select(dispatcher => dispatcher.Description)
                    .ToList(),
                // Receivers = this
                //     .core.dispatchReceivers.Select(receiver => receiver.Description)
                //     .ToList(),
                Resources = this.core.resources.ResourceInfo,
                Info = new() { { "Storage Volume", this.core.resources.UsedVolumeString } },
                Alerts = this.core.Alerts.Count == 0 ? null : this.core.Alerts,
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

            this.core.dispatchers = new List<DispatchComponentCore>()
            {
                new(
                    this.core,
                    this.core.battery,
                    this.core.resources,
                    gameContent,
                    // Deploy...
                    DispatchComponentCore.Verbs.Deploy.ToString(),
                    // ...mining drill...
                    FactoryGameContent.Spawnables.MiningDrill.ToString(),
                    // ...to coal.
                    this.core.targetType
                ),
                new(
                    this.core,
                    this.core.battery,
                    this.core.resources,
                    gameContent,
                    // Deliver...
                    DispatchComponentCore.Verbs.Deliver.ToString(),
                    // ...coal...
                    FactoryGameContent.Resources.Coal.ToString(),
                    // ...to me.
                    DispatchComponentCore.Keywords.Me.ToString()
                ),
            };

            // this.core.dispatchReceivers = new List<DispatchReceiverComponentCore>()
            // {
            //     new(
            //         this.core,
            //         this.core.resources,
            //         DispatchComponentCore.Verbs.Stockpile.ToString(),
            //         FactoryGameContent.Resources.Coal.ToString()
            //     ),
            // };
            this.core.powerLine = new PowerLineComponentCore(
                this.core,
                FactoryGameContent.Spawnables.PowerLines.ToString()
            );
        }

        public override void Tick(GameController gameController)
        {
            base.Tick(gameController);
            foreach (ResourceInserterComponentCore inserter in this.core.resourceInserters)
            {
                inserter.Insert(this.core, gameController.core);
            }
            foreach (DispatchComponentCore dispatcher in this.core.dispatchers)
            {
                this.core.Alerts = dispatcher.Tick(gameController.core);
            }
            this.core.power.GeneratePower();
            this.core.battery.Balance(this.core, gameController.core);
            this.core.powerLine.Tick(gameController.core);
        }
    }
}
