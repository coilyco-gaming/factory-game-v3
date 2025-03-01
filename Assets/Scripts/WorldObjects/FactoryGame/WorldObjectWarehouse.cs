using System;
using Assets.Scripts.Components.Core;
using Assets.Scripts.Core;
using Assets.Scripts.Unity;
using Assets.Scripts.WorldObjects.Unity;

namespace Assets.Scripts.WorldObjects.FactoryGame
{
    public class WorldObjectWarehouse : WorldObject
    {
        private static uint totalVolumeCapacity = 5000;
        private static uint totalWeightCapacity = uint.MaxValue;

        public override void Instantiate(
            GameController gameController,
            GameControllerCore.SpawnQueueItem spawnQueueItem
        )
        {
            base.Instantiate(gameController, spawnQueueItem);
            this.Resources.Instantiate(
                weightCapacity: WorldObjectWarehouse.totalWeightCapacity,
                volumeCapacity: WorldObjectWarehouse.totalVolumeCapacity
            );
        }

        protected override Func<StatusDataComponentCore.StatusData> GetStatusData()
        {
            return () =>
            {
                StatusDataComponentCore.StatusData statusData = new()
                {
                    Name = this.WorldObjectType,
                    Info = this.Resources.ResourceInfo,
                };
                statusData.Info["Storage Volume"] = this.Resources.UsedVolumeString;
                return statusData;
            };
        }
    }
}
