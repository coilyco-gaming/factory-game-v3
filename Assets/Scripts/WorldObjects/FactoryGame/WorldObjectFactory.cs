using System;
using System.Collections.Generic;
using System.Linq;
using Assets.Scripts.Components.Core;
using Assets.Scripts.Core;
using Assets.Scripts.Unity;
using Assets.Scripts.WorldObjects.Unity;

namespace Assets.Scripts.WorldObjects.FactoryGame
{
    public class WorldObjectFactory : WorldObject
    {
        public string productType;
        private static uint totalVolumeCapacity = 1000;
        private static uint totalWeightCapacity = uint.MaxValue;
        private static uint totalBatteryCapacity = 1000;
        private static uint insertionRate = 5;

        public override void Instantiate(
            GameController gameController,
            GameControllerCore.SpawnQueueItem spawnQueueItem
        )
        {
            base.Instantiate(gameController, spawnQueueItem);

            this.core.Resources = new(
                new FactoryGameContent(),
                weightCapacity: WorldObjectFactory.totalWeightCapacity,
                volumeCapacity: WorldObjectFactory.totalVolumeCapacity
            );

            this.core.Battery = new(capacity: WorldObjectFactory.totalBatteryCapacity);

            this.core.Production = new ProductionComponentCore(
                new FactoryGameContent(),
                this.core.Resources,
                FactoryGameContent.Products.BuildingMaterials.ToString()
            );

            List<string> ingredients = new FactoryGameContent()
                .Items[this.productType]
                .Ingredients.Keys.ToList();

            this.core.Inserters = new();
            foreach (string ingredient in ingredients)
            {
                this.core.Inserters.Add(
                    new(
                        this.core.Battery,
                        this.core.Resources,
                        ingredient,
                        WorldObjectFactory.insertionRate
                    )
                );
            }
        }

        public override void Tick(GameController gameController)
        {
            base.Tick(gameController);
            foreach (InserterComponentCore inserter in this.core.Inserters)
            {
                inserter.Insert(this.core, gameController.core);
            }
            this.core.Battery.Balance(this.core, gameController.core);
            this.core.Production.Produce();
        }

        protected override Func<StatusDataComponentCore.StatusData> GetStatusData()
        {
            return () =>
            {
                StatusDataComponentCore.StatusData statusData = new()
                {
                    Name = this.WorldObjectType,
                    Info = this.core.Resources.ResourceInfo,
                };
                statusData.Info["Product"] = this.productType;
                statusData.Info["Storage Volume"] = this.core.Resources.UsedVolumeString;
                statusData.Info["Energy"] = this.core.Battery.PercentEnergyStatus;
                return statusData;
            };
        }
    }
}
