using System;
using System.Collections.Generic;
using Assets.Scripts.Components.Unity;
using Unity.VisualScripting;

namespace Assets.Scripts.WorldObjects.FactoryGame
{
    public class WorldObjectOre : WorldObject
    {
        public ResourcesComponent Resources { get; set; }
        public int Amount { get; set; }

        public override void PostInstantiate(
            GameController gameController,
            GameController.SpawnQueueItem spawnQueueItem
        )
        {
            base.PostInstantiate(gameController, spawnQueueItem);
            this.Resources = this.AddComponent<ResourcesComponent>();
            this.Resources.Instantiate(
                this.Amount,
                new Dictionary<string, int> { { this.WorldObjectType, this.Amount } }
            );
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
