using System;
using System.Collections.Generic;
using Assets.Scripts.Components.Unity;
using Assets.Scripts.Unity;
using Assets.Scripts.WorldObjects.Unity;
using Unity.VisualScripting;

namespace Assets.Scripts.WorldObjects.FactoryGame
{
    public class WorldObjectFactory : WorldObject
    {
        private static uint totalVolumeCapacity = 1000;
        private static uint totalWeightCapacity = uint.MaxValue;
        private static uint totalBatteryCapacity = 1000;
        private static uint insertionRate = 5;
        private List<InserterComponent> inserters;

        public override void Instantiate(
            GameController gameController,
            GameController.SpawnQueueItem spawnQueueItem
        )
        {
            base.Instantiate(gameController, spawnQueueItem);
            this.Resources.Instantiate(
                weightCapacity: WorldObjectFactory.totalWeightCapacity,
                volumeCapacity: WorldObjectFactory.totalVolumeCapacity
            );

            // Iron and Copper inserters, for building robots.
            this.inserters = new List<InserterComponent>()
            {
                this.AddComponent<InserterComponent>(),
                this.AddComponent<InserterComponent>(),
                this.AddComponent<InserterComponent>(),
            };
            this.inserters[0]
                .Instantiate(
                    this.Resources,
                    FactoryGameContent.Resources.Iron.ToString(),
                    WorldObjectFactory.insertionRate
                );
            this.inserters[1]
                .Instantiate(
                    this.Resources,
                    FactoryGameContent.Resources.Stone.ToString(),
                    WorldObjectFactory.insertionRate
                );
            this.inserters[2]
                .Instantiate(
                    this.Resources,
                    FactoryGameContent.Resources.Copper.ToString(),
                    WorldObjectFactory.insertionRate
                );

            this.Battery.Instantiate(capacity: WorldObjectFactory.totalBatteryCapacity);
        }

        public override void Tick(GameController gameController)
        {
            base.Tick(gameController);
            foreach (InserterComponent inserter in this.inserters)
            {
                if (inserter != null)
                {
                    inserter.Insert(this, gameController);
                }
            }
            this.Battery.Balance(this, gameController);
        }

        protected override Func<StatusDataComponent.StatusData> GetStatusData()
        {
            return () =>
            {
                StatusDataComponent.StatusData statusData = new()
                {
                    Name = this.WorldObjectType,
                    Info = this.Resources.ResourceInfo,
                };
                statusData.Info["Storage Volume"] = this.Resources.UsedVolumeString;
                statusData.Info["Energy"] = this.Battery.PercentEnergyStatus;
                return statusData;
            };
        }
    }
}
