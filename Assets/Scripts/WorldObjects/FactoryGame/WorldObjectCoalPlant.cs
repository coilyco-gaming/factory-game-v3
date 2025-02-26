using System;
using System.Collections.Generic;
using Assets.Scripts.Components.Unity;
using Unity.VisualScripting;

namespace Assets.Scripts.WorldObjects.FactoryGame
{
    public class WorldObjectCoalPlant : WorldObject
    {
        private static uint insertionRate = 5;
        private static uint totalResourceCapacity = 100;
        private static uint totalBatteryCapacity = 5000;
        private static uint powerBurnRate = 5;
        private static uint powerGainRate = 100;
        private List<InserterComponent> inserters;
        private PowerComponent power;

        public override void Instantiate(
            GameController gameController,
            GameController.SpawnQueueItem spawnQueueItem
        )
        {
            base.Instantiate(gameController, spawnQueueItem);
            this.Resources.Instantiate(WorldObjectCoalPlant.totalResourceCapacity);

            this.inserters = new List<InserterComponent>()
            {
                this.AddComponent<InserterComponent>(),
                this.AddComponent<InserterComponent>(),
            };
            this.inserters[0]
                .Instantiate(
                    this.Resources,
                    FactoryGameController.Ores.Coal.ToString(),
                    WorldObjectCoalPlant.insertionRate
                );

            this.Battery.Instantiate(capacity: WorldObjectCoalPlant.totalBatteryCapacity);

            this.power = this.AddComponent<PowerComponent>();
            this.power.Instantiate(
                this.Battery,
                this.Resources,
                FactoryGameController.Ores.Coal.ToString(),
                burnRate: WorldObjectCoalPlant.powerBurnRate,
                gainRate: WorldObjectCoalPlant.powerGainRate
            );
        }

        public override void Tick(GameController gameController)
        {
            base.Tick(gameController);
            foreach (InserterComponent inserter in this.inserters)
            {
                inserter.Insert(this, gameController);
            }
            this.power.GeneratePower();
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
                statusData.Info["Energy"] = this.Battery.PercentEnergyStatus;
                return statusData;
            };
        }
    }
}
