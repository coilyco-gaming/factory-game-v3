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
        public uint totalVolumeCapacity = 5000;
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
                this.core.targetType
            );

            this.core.dispatch = new(
                this.core,
                this.core.battery,
                FactoryGameContent.DispatchVerbs.Deploy.ToString(), // Deploy
                FactoryGameContent.Spawnables.MiningDrill.ToString(), // A mining drill
                this.core.targetSubType // To iron ore
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
            {
                StatusDataComponentCore.StatusData statusData = new()
                {
                    Name = this.WorldObjectType,
                    Info = this.core.resources.ResourceInfo,
                };
                if (this.core.production.PercentCraftProgress != 0)
                {
                    statusData.Info["Progress"] = this.core.production.PrecentProgressStatus;
                }
                statusData.Info["Product"] = this.core.targetType;
                statusData.Info["Storage Volume"] = this.core.resources.UsedVolumeString;
                statusData.Info["Energy"] = this.core.battery.PercentEnergyStatus;
                return statusData;
            };
        }
    }
}
