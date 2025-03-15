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
                Receivers = this
                    .core.dispatchReceivers.Select(receiver => receiver.Description)
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
                gameContent,
                weightCapacity: uint.MaxValue,
                volumeCapacity: this.totalVolumeCapacity
            );

            this.core.battery = new(capacity: this.totalBatteryCapacity);

            List<string> ingredients = gameContent
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
                    // Retrieve...
                    DispatchComponentCore.Verbs.Retrieve.ToString(),
                    // ...< iron bar | copper bar >...
                    this.core.targetType,
                    // ...from me.
                    DispatchComponentCore.Keywords.Me.ToString()
                ),
            };

            this.core.dispatchReceivers ??= new();
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
                // this.core.dispatchReceivers.Add(
                //     new(
                //         this.core,
                //         this.core.resources,
                //         // Stockpile...
                //         DispatchComponentCore.Verbs.Stockpile.ToString(),
                //         // ...< ingredient >
                //         ingredient
                //     )
                // );
            }
            this.core.powerLine = new PowerLineComponentCore(
                this.core,
                FactoryGameContent.Spawnables.PowerLines.ToString()
            );
        }

        public override void Tick(GameController gameController)
        {
            base.Tick(gameController);
            this.core.battery.Balance(this.core, gameController.core);
            this.core.production.Produce();
            foreach (DispatchComponentCore dispatcher in this.core.dispatchers)
            {
                this.core.Alerts = dispatcher.Tick(gameController.core);
            }
            foreach (ResourceInserterComponentCore inserter in this.core.resourceInserters)
            {
                inserter.Insert(this.core, gameController.core);
            }
            this.core.powerLine.Tick(gameController.core);
        }
    }
}
