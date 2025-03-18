using System;
using System.Collections.Generic;
using System.Linq;
using Assets.Scripts.Components.Core;
using Assets.Scripts.Core;
using Assets.Scripts.Unity;

namespace Assets.Scripts.WorldObjects.FactoryGame
{
    [Serializable]
    public class WorldObjectFactory : WorldObject
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
                    { "Progress", this.core.production.PrecentProgressStatus },
                    { "Storage Volume", this.core.resources.UsedVolumeString },
                },
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
                this.core.targetType
            );

            this.core.dispatchers = new();

            bool spawnable = gameContent.Items[this.core.targetType].CanSpawnGameObject;
            if (spawnable)
            {
                // Retrieve is for items that can be deployed
                this.core.dispatchers.Add(
                    new(
                        this.core,
                        this.core.battery,
                        this.core.resources,
                        gameContent,
                        // Retrieve...
                        DispatchComponentCore.Verbs.Retrieve.ToString(),
                        // ...< product >...
                        this.core.targetType,
                        // ...from me.
                        DispatchComponentCore.Keywords.Me.ToString()
                    )
                );
            }
            else
            {
                // Collect is for stockpiling
                this.core.dispatchers.Add(
                    new(
                        this.core,
                        this.core.battery,
                        this.core.resources,
                        gameContent,
                        // Collect...
                        DispatchComponentCore.Verbs.Collect.ToString(),
                        // ...< product >...
                        this.core.targetType,
                        // ...from me.
                        DispatchComponentCore.Keywords.Me.ToString()
                    )
                );
            }

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
            this.core.Alerts = this.core.battery.Tick(gameController.core);
            this.core.Alerts = this.core.production.Tick(gameController.core);
            foreach (DispatchComponentCore dispatcher in this.core.dispatchers)
            {
                this.core.Alerts = this.core.Alerts = dispatcher.Tick(gameController.core);
            }
            foreach (ResourceInserterComponentCore inserter in this.core.resourceInserters)
            {
                this.core.Alerts = inserter.Tick(gameController.core);
            }
            this.core.Alerts = this.core.powerLine.Tick(gameController.core);
        }
    }
}
