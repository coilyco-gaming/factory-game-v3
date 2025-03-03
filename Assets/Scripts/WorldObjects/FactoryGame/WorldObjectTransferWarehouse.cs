using System;
using Assets.Scripts.Components.Core;
using Assets.Scripts.Core;
using Assets.Scripts.Unity;
using Assets.Scripts.WorldObjects.Unity;

namespace Assets.Scripts.WorldObjects.FactoryGame
{
    public class WorldObjectTransferWarehouse : WorldObject
    {
        private static uint totalBatteryCapacity = 10000;
        private static uint totalVolumeCapacity = 10000;
        private static uint totalWeightCapacity = uint.MaxValue;

        public override void Instantiate(
            GameController gameController,
            GameControllerCore.SpawnQueueItem spawnQueueItem
        )
        {
            base.Instantiate(gameController, spawnQueueItem);
            this.core.Resources = new(
                new FactoryGameContent(),
                weightCapacity: WorldObjectTransferWarehouse.totalWeightCapacity,
                volumeCapacity: WorldObjectTransferWarehouse.totalVolumeCapacity
            );
            this.core.Battery = new(capacity: WorldObjectTransferWarehouse.totalBatteryCapacity);
            this.core.TransferHub = new(gameController.core, this.core, this.core.Battery);
        }

        public override void Tick(GameController gameController)
        {
            base.Tick(gameController);
            this.core.TransferHub.Balance();
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
                statusData.Info["Energy"] = this.core.Battery.PercentEnergyStatus;
                return statusData;
            };
        }
    }
}
