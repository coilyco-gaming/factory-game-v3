using System;
using Assets.Scripts.Components.Core;
using Assets.Scripts.Core;
using Assets.Scripts.ScriptableObject;
using Assets.Scripts.Unity;
using Assets.Scripts.WorldObjects.Unity;

namespace Assets.Scripts.WorldObjects.FactoryGame
{
    [Serializable]
    public class WorldObjectOre : WorldObject
    {
        public override void Instantiate(SpawnQueueItem spawnQueueItem)
        {
            base.Instantiate(spawnQueueItem);
            this.core.resources = new(
                new FactoryGameContent(),
                weightCapacity: uint.MaxValue,
                volumeCapacity: uint.MaxValue
            );
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
                new()
                {
                    Name = Util.HumanizedString(this.WorldObjectType),
                    Resources = this.core.resources.ResourceInfo,
                };
        }
    }
}
