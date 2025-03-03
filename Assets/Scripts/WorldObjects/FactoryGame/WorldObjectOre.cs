using System;
using Assets.Scripts.Components.Core;
using Assets.Scripts.Core;
using Assets.Scripts.Unity;
using Assets.Scripts.WorldObjects.Unity;

namespace Assets.Scripts.WorldObjects.FactoryGame
{
    [Serializable]
    public class WorldObjectOre : WorldObject
    {
        public uint amount;

        public override void Instantiate(
            GameController gameController,
            GameControllerCore.SpawnQueueItem spawnQueueItem
        )
        {
            base.Instantiate(gameController, spawnQueueItem);
            this.core.resources = new(
                new FactoryGameContent(),
                weightCapacity: this.amount,
                volumeCapacity: this.amount
            );
            this.core.resources.CreateResources(this.WorldObjectType, this.amount);
        }

        public override void Tick(GameController gameController)
        {
            base.Tick(gameController);

            // If the ore is empty, delete it.
            if (!this.core.resources.HasResources)
            {
                gameController.QueueForDeletion(
                    new GameControllerCore.DeletionQueueItem(this.core, this.GridPosition)
                );
            }
        }

        protected override Func<StatusDataComponentCore.StatusData> GetStatusData()
        {
            return () =>
                new StatusDataComponentCore.StatusData()
                {
                    Name = this.WorldObjectType,
                    Info = this.core.resources.ResourceInfo,
                };
        }
    }
}
