using System;
using System.Collections.Generic;
using System.Linq;
using Assets.Scripts.Components.Core;
using Assets.Scripts.Core;
using Assets.Scripts.Unity;
using Assets.Scripts.WorldObjects.Unity;

namespace Assets.Scripts.WorldObjects.FactoryGame
{
    [Serializable]
    public class WorldObjectFoundry : WorldObject
    {
        public uint totalVolumeCapacity = 10000;
        public uint totalBatteryCapacity = 1000;
        public uint insertionRate = 5;

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
                Alerts = this.core.Alerts.Count == 0 ? null : this.core.Alerts,
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

            List<string> ingredients = new FactoryGameContent()
                .Items[this.core.targetType]
                .Ingredients.Keys.ToList();

            this.core.resourceInserters = new();
            foreach (string ingredient in ingredients)
            {
                this.core.resourceInserters.Add(
                    new(this.core.battery, this.core.resources, ingredient, this.insertionRate)
                );
            }

            this.core.production = new ProductionComponentCore(
                new FactoryGameContent(),
                this.core.resources,
                this.core.battery,
                this.core.resourceInserters,
                this.core.targetType // < iron bar | copper bar >
            );

            this.core.dispatchers = new List<DispatchComponentCore>
            {
                // TODO: only dispatch when you don't have the resource
                new(
                    this.core,
                    this.core.battery,
                    this.core.resources,
                    new FactoryGameContent(),
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
                    new FactoryGameContent(),
                    // Retrieve...
                    DispatchComponentCore.Verbs.Retrieve.ToString(),
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
                        new FactoryGameContent(),
                        // Deliver...
                        DispatchComponentCore.Verbs.Deliver.ToString(),
                        // ...ingredient...
                        ingredient,
                        // ...to me.
                        DispatchComponentCore.Keywords.Me.ToString()
                    )
                );
            }
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
            this.core.battery.Balance(this.core, gameController.core);
            this.core.production.Produce();
        }
    }
}
