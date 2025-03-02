using System;
using System.Collections.Generic;
using Assets.Scripts.Components.Core;
using Assets.Scripts.Core;
using Assets.Scripts.Unity;
using Assets.Scripts.WorldObjects.Unity;

namespace Assets.Scripts.WorldObjects.FactoryGame
{
    public class WorldObjectCoalPlant : WorldObject
    {
        private static uint totalVolumeCapacity = 1000;
        private static uint totalWeightCapacity = uint.MaxValue;
        private static uint totalBatteryCapacity = 5000;
        private static uint insertionRate = 5;
        private static uint powerBurnRate = 5;
        private static uint powerGainRate = 100;

        public override void Instantiate(
            GameController gameController,
            GameControllerCore.SpawnQueueItem spawnQueueItem
        )
        {
            base.Instantiate(gameController, spawnQueueItem);
            this.core.Resources = new(
                new FactoryGameContent(),
                weightCapacity: WorldObjectCoalPlant.totalWeightCapacity,
                volumeCapacity: WorldObjectCoalPlant.totalVolumeCapacity
            );

            this.core.Inserters = new List<InserterComponentCore>()
            {
                new(
                    this.core.Resources,
                    FactoryGameContent.Resources.Coal.ToString(),
                    WorldObjectCoalPlant.insertionRate
                ),
                new(
                    this.core.Resources,
                    FactoryGameContent.Resources.Coal.ToString(),
                    WorldObjectCoalPlant.insertionRate
                ),
            };

            this.core.Battery = new(capacity: WorldObjectCoalPlant.totalBatteryCapacity);

            // this.core.Power = new PowerComponentCore();
            // this.core.Power.Instantiate(
            //     this.core.Battery,
            //     this.core.Resources,
            //     FactoryGameContent.Resources.Coal.ToString(),
            //     burnRate: WorldObjectCoalPlant.powerBurnRate,
            //     gainRate: WorldObjectCoalPlant.powerGainRate
            // );
        }

        public override void Tick(GameController gameController)
        {
            base.Tick(gameController);
            foreach (InserterComponentCore inserter in this.core.Inserters)
            {
                inserter?.Insert(this.core, gameController.core);
            }
            // this.core.Power.GeneratePower();
            this.core.Battery.Balance(this, gameController);
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
