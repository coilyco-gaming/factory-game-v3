using System;
using System.Collections.Generic;
using System.Linq;
using Assets.Scripts.Components.Core;
using Assets.Scripts.Core;
using Assets.Scripts.Unity;

namespace Assets.Scripts.WorldObjects.FactoryGame
{
    [Serializable]
    public class WorldObjectFoundry : WorldObject
    {
        private uint totalVolumeCapacity = 10000;
        private uint totalBatteryCapacity = 1000;
        private uint insertionRate = 5;

        public override StatusDataComponentCore StatusData =>
            new()
            {
                Name = Util.HumanizedString(this.WorldObjectType),
                Energy = this.core.battery.PercentEnergyStatus,
                Dispatchers = this
                    .core.dispatchers.Select(dispatcher => dispatcher.Description)
                    .ToList(),
                Resources = this.core.resources.ResourceInfo,
                Info = new()
                {
                    { "Outputs", Util.HumanizedString(this.core.targetType).ToLower() },
                    { "Storage Volume", this.core.resources.UsedVolumeString },
                },
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

            this.core.battery = new(this.core, capacity: this.totalBatteryCapacity);

            List<string> ingredients = gameContent
                .Items[this.core.targetType]
                .Ingredients.Keys.ToList();

            this.core.resourceInserters = new();
            foreach (string ingredient in ingredients)
            {
                this.core.resourceInserters.Add(
                    new(
                        this.core,
                        this.core.battery,
                        this.core.resources,
                        ingredient,
                        this.insertionRate
                    )
                );
            }

            this.core.production = new ProductionComponentCore(
                this.core,
                gameContent,
                this.core.resources,
                this.core.battery,
                this.core.resourceInserters,
                this.core.targetType // < iron bar | copper bar >
            );

            this.core.dispatchers = new List<DispatchComponentCore>
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
                    // ...to < iron ore | copper ore >.
                    this.core.targetSubType
                ),
                new(
                    this.core,
                    this.core.battery,
                    this.core.resources,
                    gameContent,
                    // Collect...
                    DispatchComponentCore.Verbs.Collect.ToString(),
                    // ...< iron bar | copper bar >...
                    this.core.targetType,
                    // ...from me.
                    DispatchComponentCore.Keywords.Me.ToString()
                ),
            };

            foreach (string ingredient in ingredients)
            {
                this.core.dispatchers.Add(
                    new(
                        this.core,
                        this.core.battery,
                        this.core.resources,
                        gameContent,
                        // Deliver...
                        DispatchComponentCore.Verbs.Deliver.ToString(),
                        // ...< ingredient >...
                        ingredient,
                        // ...to me.
                        DispatchComponentCore.Keywords.Me.ToString()
                    )
                );
            }
            this.core.powerLine = new PowerLineComponentCore(
                this.core,
                FactoryGameContent.Spawnables.PowerLines.ToString()
            );
        }

        public override void Tick(GameController gameController)
        {
            base.Tick(gameController);
            this.core.CreateAlert(gameController.core, this.core.battery.Tick(gameController.core));
            this.core.CreateAlert(
                gameController.core,
                this.core.production.Tick(gameController.core)
            );
            foreach (DispatchComponentCore dispatcher in this.core.dispatchers)
            {
                this.core.CreateAlert(gameController.core, dispatcher.Tick(gameController.core));
            }
            foreach (ResourceInserterComponentCore inserter in this.core.resourceInserters)
            {
                this.core.CreateAlert(gameController.core, inserter.Tick(gameController.core));
            }
            this.core.CreateAlert(
                gameController.core,
                this.core.powerLine.Tick(gameController.core)
            );
        }
    }
}
