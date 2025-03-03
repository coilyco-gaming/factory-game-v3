using System;
using Assets.Scripts.Components.Core;
using Assets.Scripts.Core;
using Assets.Scripts.Unity;
using Assets.Scripts.WorldObjects.Unity;

namespace Assets.Scripts.WorldObjects.FactoryGame
{
    public class WorldObjectStorageWarehouse : WorldObject
    {
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
                weightCapacity: WorldObjectStorageWarehouse.totalWeightCapacity,
                volumeCapacity: WorldObjectStorageWarehouse.totalVolumeCapacity
            );
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
                return statusData;
            };
        }
    }
}
