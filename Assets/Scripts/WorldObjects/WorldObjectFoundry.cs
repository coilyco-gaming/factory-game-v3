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
                Guid = this.core.guid,
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

            this.core.battery = new(capacity: this.totalBatteryCapacity);

            List<string> ingredients = gameContent
                .Items[this.core.targetType]
                .Ingredients.Keys.ToList();

            this.core.resourceInserters = new();
            foreach (string ingredient in ingredients)
            {
                this.core.resourceInserters.Add(new(ingredient, this.insertionRate));
            }

            this.core.production = new ProductionComponentCore(
                gameContent,
                this.core.targetType // < iron bar | copper bar >
            );
            this.core.production.SetReservedCapacity(this.core.resources);
            this.core.production.SetInserterResourceTypes(this.core.resourceInserters);

            this.core.dispatchers = new List<DispatchComponentCore>
            {
                new(
                    gameContent,
                    this.core,
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
                        gameContent,
                        this.core,
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
                FactoryGameContent.Spawnables.PowerLines.ToString()
            );
        }

        public override void Tick(GameController gameController)
        {
            base.Tick(gameController);
            this.core.CreateAlert(
                gameController.core,
                this.core.battery.Tick(gameController.core, this.core)
            );
            this.core.CreateAlert(
                gameController.core,
                this.core.production.Tick(gameController.core, this.core)
            );
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
                this.core.powerLine.Tick(gameController.core, this.core)
            );
        }
    }
}
