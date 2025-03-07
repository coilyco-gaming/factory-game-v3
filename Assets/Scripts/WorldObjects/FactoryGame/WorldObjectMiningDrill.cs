using System;
using Assets.Scripts.Components.Core;
using Assets.Scripts.ScriptableObject;
using Assets.Scripts.Unity;
using Assets.Scripts.WorldObjects.Unity;

namespace Assets.Scripts.WorldObjects.FactoryGame
{
    [Serializable]
    public class WorldObjectMiningDrill : WorldObject
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
        }

        public override void Tick(GameController gameController)
        {
            base.Tick(gameController);
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
                statusData.Info["Product"] = this.core.targetType;
                statusData.Info["Storage Volume"] = this.core.resources.UsedVolumeString;
                statusData.Info["Energy"] = this.core.battery.PercentEnergyStatus;
                return statusData;
            };
        }
    }
}
