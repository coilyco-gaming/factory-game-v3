using System;
using System.Collections.Generic;
using Assets.Scripts.Components.Core;
using Assets.Scripts.Core;
using Assets.Scripts.Unity;
using Assets.Scripts.WorldObjects.Unity;

namespace Assets.Scripts.WorldObjects.FactoryGame
{
    public class WorldObjectFactory : WorldObject
    {
        public string productType;
        public uint productQuantity;
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

            this.core.Production = new ProductionComponentCore(
                new FactoryGameContent(),
                this.core.Resources,
                FactoryGameContent.Products.BuildingMaterials.ToString()
            );

            this.core.Inserters = new List<InserterComponentCore>() { new(), new(), new(), new() };
            this.core.Inserters[0]
                .Instantiate(
                    this.core.Resources,
                    FactoryGameContent.Resources.Iron.ToString(),
                    WorldObjectFactory.insertionRate
                );
            this.core.Inserters[1]
                .Instantiate(
                    this.core.Resources,
                    FactoryGameContent.Resources.Stone.ToString(),
                    WorldObjectFactory.insertionRate
                );
            this.core.Inserters[2]
                .Instantiate(
                    this.core.Resources,
                    FactoryGameContent.Resources.Copper.ToString(),
                    WorldObjectFactory.insertionRate
                );

            // this.core.Battery.Instantiate(capacity: WorldObjectFactory.totalBatteryCapacity);
        }

        public override void Tick(GameController gameController)
        {
            base.Tick(gameController);
            foreach (InserterComponentCore inserter in this.core.Inserters)
            {
                inserter?.Insert(this.core, gameController.core);
            }
            // this.core.Battery.Balance(this, gameController);
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
                statusData.Info["Storage Volume"] = this.core.Resources.UsedVolumeString;
                // statusData.Info["Energy"] = this.core.Battery.PercentEnergyStatus;
                return statusData;
            };
        }
    }
}
