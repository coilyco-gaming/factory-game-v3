using System;
using Assets.Scripts.Components.Core;
using Assets.Scripts.Core;
using Assets.Scripts.Unity;
using Assets.Scripts.WorldObjects.Unity;

namespace Assets.Scripts.WorldObjects.FactoryGame
{
    [Serializable]
    public class WorldObjectTransferWarehouse : WorldObject
    {
        public uint totalBatteryCapacity = 10000;
        public uint totalVolumeCapacity = 1000;
        public uint totalWeightCapacity = uint.MaxValue;

        public override void Instantiate(
            GameController gameController,
            GameControllerCore.SpawnQueueItem spawnQueueItem
        )
        {
            base.Instantiate(gameController, spawnQueueItem);
            this.core.battery = new(capacity: this.totalBatteryCapacity);
            this.core.transferHub = new(
                new FactoryGameContent(),
                gameController.core,
                this.core,
                this.core.battery
            );
        }

        public override void Tick(GameController gameController)
        {
            base.Tick(gameController);
            this.core.transferHub.Balance();
        }

        protected override Func<StatusDataComponentCore.StatusData> GetStatusData()
        {
            return () =>
            {
                StatusDataComponentCore.StatusData statusData = new()
                {
                    Name = this.WorldObjectType,
                    Info = new() { ["Energy"] = this.core.battery.PercentEnergyStatus },
                };
                return statusData;
            };
        }
    }
}
