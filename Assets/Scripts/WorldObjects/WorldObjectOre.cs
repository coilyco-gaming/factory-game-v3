using System;
using Assets.Scripts.Components.Core;
using Assets.Scripts.Core;
using Assets.Scripts.Unity;

namespace Assets.Scripts.WorldObjects.FactoryGame
{
    [Serializable]
    public class WorldObjectOre : WorldObject
    {
        public override StatusDataComponentCore StatusData =>
            new()
            {
                Name = Util.HumanizedString(this.WorldObjectType),
                Resources = this.core.resources.ResourceInfo,
            };

        public override void Instantiate(SpawnQueueItem spawnQueueItem, GameContent gameContent)
        {
            base.Instantiate(spawnQueueItem, gameContent);
            this.core.resources = new(
                gameContent,
                weightCapacity: uint.MaxValue,
                volumeCapacity: uint.MaxValue
            )
            {
                resources = spawnQueueItem.resources,
            };
        }

        public override void Tick(GameController gameController)
        {
            base.Tick(gameController);

            // If the ore is empty, delete it.
            if (!this.core.resources.HasResources)
            {
                gameController.QueueForDeletion(
                    new DeletionQueueItem(this.core, this.GridPosition)
                );
            }
        }
    }
}
