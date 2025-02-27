using System;
using Assets.Scripts.Components.Unity;
using Assets.Scripts.Unity;
using Assets.Scripts.WorldObjects.Unity;

namespace Assets.Scripts.WorldObjects.FactoryGame
{
    public class WorldObjectOre : WorldObject
    {
        public uint Amount { get; set; }

        public override void Instantiate(
            GameController gameController,
            GameController.SpawnQueueItem spawnQueueItem
        )
        {
            base.Instantiate(gameController, spawnQueueItem);
            this.Resources.Instantiate(weightCapacity: this.Amount, volumeCapacity: this.Amount);
            this.Resources.CreateResources(this.WorldObjectType, this.Amount);
        }

        public override void Tick(GameController gameController)
        {
            base.Tick(gameController);

            // If the ore is empty, delete it.
            if (!this.Resources.HasResources)
            {
                gameController.QueueForDeletion(
                    new GameController.DeletionQueueItem(this, this.GridPosition)
                );
            }
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
