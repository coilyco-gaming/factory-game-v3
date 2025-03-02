using System;
using Assets.Scripts.Components.Core;
using Assets.Scripts.Core;
using Assets.Scripts.Unity;
using Assets.Scripts.WorldObjects.Unity;

namespace Assets.Scripts.WorldObjects.FactoryGame
{
    public class WorldObjectOre : WorldObject
    {
        public uint Amount { get; set; }

        public override void Instantiate(
            GameController gameController,
            GameControllerCore.SpawnQueueItem spawnQueueItem
        )
        {
            base.Instantiate(gameController, spawnQueueItem);
            this.core.Resources = new(
                new FactoryGameContent(),
                weightCapacity: this.Amount,
                volumeCapacity: this.Amount
            );
            this.core.Resources.CreateResources(this.WorldObjectType, this.Amount);
        }

        public override void Tick(GameController gameController)
        {
            base.Tick(gameController);

            // If the ore is empty, delete it.
            if (!this.core.Resources.HasResources)
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
                    Info = this.core.Resources.ResourceInfo,
                };
        }
    }
}
