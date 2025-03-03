using System;
using System.Collections.Generic;
using Assets.Scripts.Components.Core;
using Assets.Scripts.Core;
using Assets.Scripts.Unity;
using Assets.Scripts.WorldObjects.Unity;

namespace Assets.Scripts.WorldObjects.FactoryGame
{
    [Serializable]
    public class WorldObjectCoalPlant : WorldObject
    {
        public uint totalVolumeCapacity = 1000;
        public uint totalWeightCapacity = uint.MaxValue;
        public uint totalBatteryCapacity = 5000;
        public uint insertionRate = 5;
        public uint powerBurnRate = 5;
        public uint powerGainRate = 100;

        public override void Instantiate(
            GameController gameController,
            GameControllerCore.SpawnQueueItem spawnQueueItem
        )
        {
            base.Instantiate(gameController, spawnQueueItem);
            this.core.resources = new(
                new FactoryGameContent(),
                weightCapacity: this.totalWeightCapacity,
                volumeCapacity: this.totalVolumeCapacity
            );

            this.core.battery = new(capacity: this.totalBatteryCapacity);

            this.core.inserters = new List<InserterComponentCore>()
            {
                new(
                    this.core.battery,
                    this.core.resources,
                    FactoryGameContent.Resources.Coal.ToString(),
                    this.insertionRate
                ),
                new(
                    this.core.battery,
                    this.core.resources,
                    FactoryGameContent.Resources.Coal.ToString(),
                    this.insertionRate
                ),
            };

            this.core.power = new PowerComponentCore(
                this.core.battery,
                this.core.resources,
                FactoryGameContent.Resources.Coal.ToString(),
                this.powerBurnRate,
                this.powerGainRate
            );
        }

        public override void Tick(GameController gameController)
        {
            base.Tick(gameController);
            foreach (InserterComponentCore inserter in this.core.inserters)
            {
                inserter.Insert(this.core, gameController.core);
            }
            this.core.power.GeneratePower();
            this.core.battery.Balance(this.core, gameController.core);
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
                statusData.Info["Storage Volume"] = this.core.resources.UsedVolumeString;
                statusData.Info["Energy"] = this.core.battery.PercentEnergyStatus;
                return statusData;
            };
        }
    }
}
