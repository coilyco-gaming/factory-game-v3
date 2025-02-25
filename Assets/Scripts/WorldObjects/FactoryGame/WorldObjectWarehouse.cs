using System;
using Assets.Scripts.Components.Unity;

namespace Assets.Scripts.WorldObjects.FactoryGame
{
    public class WorldObjectWarehouse : WorldObject
    {
        private static uint totalResourceCapacity = 1000;

        public override void Instantiate(
            GameController gameController,
            GameController.SpawnQueueItem spawnQueueItem
        )
        {
            base.Instantiate(gameController, spawnQueueItem);
            this.Resources.Instantiate(WorldObjectWarehouse.totalResourceCapacity);
        }

        protected override Func<StatusDataComponent.StatusData> GetStatusData()
        {
            return () =>
                new StatusDataComponent.StatusData()
                {
                    Name = this.WorldObjectType,
                    Info = this.Resources.ResourceInfo,
                };
        }
    }
}
