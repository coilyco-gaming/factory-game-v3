using System;
using System.Collections.Generic;
using System.Linq;
using Assets.Scripts.Components.Core;
using Assets.Scripts.ScriptableObject;
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

        public override void Instantiate(SpawnQueueItem spawnQueueItem)
        {
            base.Instantiate(spawnQueueItem);

            this.core.resources = new(
                new FactoryGameContent(),
                weightCapacity: uint.MaxValue,
                volumeCapacity: this.totalVolumeCapacity
            );

            this.core.battery = new(capacity: this.totalBatteryCapacity);

            List<string> ingredients = new FactoryGameContent()
                .Items[this.core.targetType]
                .Ingredients.Keys.ToList();

            this.core.inserters = new();
            foreach (string ingredient in ingredients)
            {
                this.core.inserters.Add(
                    new(this.core.battery, this.core.resources, ingredient, this.insertionRate)
                );
            }

            this.core.production = new ProductionComponentCore(
                new FactoryGameContent(),
                this.core.resources,
                this.core.battery,
                this.core.inserters,
                this.core.targetType // Iron bar
            );

            this.core.dispatch = new(
                this.core,
                this.core.battery,
                // Deploy...
                DispatchComponentCore.Verbs.Deploy.ToString(),
                // ...mining drill...
                FactoryGameContent.Spawnables.MiningDrill.ToString(),
                // ...to < iron ore | copper ore >.
                this.core.targetSubType
            );
        }

        public override void Tick(GameController gameController)
        {
            base.Tick(gameController);
            foreach (InserterComponentCore inserter in this.core.inserters)
            {
                inserter.Insert(this.core, gameController.core);
            }
            this.core.battery.Balance(this.core, gameController.core);
            this.core.production.Produce();
            this.core.dispatch.Tick(gameController.core);
        }

        protected override Func<StatusDataComponentCore.StatusData> GetStatusData()
        {
            return () =>
                new()
                {
                    Name = Util.HumanizedString(this.WorldObjectType),
                    Resources = this.core.resources.ResourceInfo,
                    Info = new()
                    {
                        { "Outputs", Util.HumanizedString(this.core.targetType).ToLower() },
                        { "Dispatch", this.core.dispatch.Description },
                        { "Storage Volume", this.core.resources.UsedVolumeString },
                        { "Energy", this.core.battery.PercentEnergyStatus },
                    },
                };
        }
    }
}
