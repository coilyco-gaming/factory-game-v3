namespace Assets.Scripts.Components.Core
{
    using System;
    using System.Linq;
    using Assets.Scripts.Core;
    using Assets.Scripts.Unity;
    using EpPathFinding.cs;

    public class MovementComponentCore
    {
        private WorldObject worldObject;

        public MovementComponentCore(WorldObject worldObject)
        {
            this.worldObject = worldObject;
        }

        public void Tick(GameController gameController)
        {
            if (
                this.worldObject.core.dispatchReceivers.Count != 0
                && this.worldObject.core.dispatchReceivers.First().targetPosition == null
            )
            {
                // We have no target position
                return;
            }

            System.Numerics.Vector2 start = this.worldObject.GridPosition;
            System.Numerics.Vector2 end = this
                .worldObject.core.dispatchReceivers.First()
                .targetPosition.Value;

            // Determine if we are already close enough
            float xDiff = Math.Abs(start.X - end.X);
            float yDiff = Math.Abs(start.Y - end.Y);
            double distance = Math.Sqrt(Math.Pow(xDiff, 2) + Math.Pow(yDiff, 2));
            if (distance < 1.5d)
            {
                // We are close enough
                return;
            }

            // Get a movement vector to the next position on our path
            System.Numerics.Vector2? movement = PathfindingComponentCore.GetPosition(
                start: start,
                end: end,
                grid: gameController.Map.Grid.Clone() as StaticGrid
            );

            if (movement == null)
            {
                // No available path
                return;
            }

            // Queue up movement
            System.Numerics.Vector2 newPosition = new(
                this.worldObject.GridPosition.X + movement.Value.X,
                this.worldObject.GridPosition.Y + movement.Value.Y
            );
            gameController.QueueForMovement(
                new MovementQueueItem(
                    this.worldObject.GridPosition,
                    newPosition,
                    this.worldObject.core
                )
            );
        }
    }
}
