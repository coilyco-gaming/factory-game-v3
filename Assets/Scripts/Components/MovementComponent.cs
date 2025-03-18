namespace Assets.Scripts.Components.Core
{
    using System;
    using System.Collections.Generic;
    using System.Diagnostics;
    using System.Linq;
    using Assets.Scripts.Core;
    using EpPathFinding.cs;
    using UnityEngine;

    [Serializable]
    public class MovementComponentCore
    {
        private WorldObjectCore worldObject;

        public MovementComponentCore(WorldObjectCore worldObject)
        {
            this.worldObject = worldObject;
        }

        public List<Dictionary<uint, string>> Tick(GameControllerCore gameController)
        {
            using Activity activity = gameController.backref.ActivitySource.StartActivity(
                this.GetType().Name
            );
            activity.SetTag("WorldObjectType", this.worldObject.worldObjectType);

            if (
                this.worldObject.dispatchReceivers.Count != 0
                && this.worldObject.dispatchReceivers.First().targetPosition == null
            )
            {
                // We have no target position
                return new()
                {
                    new() { { gameController.backref.TickCount, "no target position" } },
                };
            }

            System.Numerics.Vector2 start = this.worldObject.GridPosition;
            System.Numerics.Vector2 end = this
                .worldObject.dispatchReceivers.First()
                .targetPosition.Value;

            // Determine if we are already close enough
            float xDiff = Math.Abs(start.X - end.X);
            float yDiff = Math.Abs(start.Y - end.Y);
            double distance = Math.Sqrt(Math.Pow(xDiff, 2) + Math.Pow(yDiff, 2));
            if (distance < 1.5d)
            {
                // We are close enough
                return new();
            }

            // Get a movement vector to the next position on our path
            System.Numerics.Vector2? movement = PathfindingComponentCore.GetPosition(
                start: start,
                end: end,
                grid: gameController.backref.Map.Grid.Clone() as StaticGrid
            );

            if (movement == null)
            {
                // No available path
                return new()
                {
                    new() { { gameController.backref.TickCount, "no available path" } },
                };
            }

            // Queue up movement
            System.Numerics.Vector2 newPosition = new(
                this.worldObject.GridPosition.X + movement.Value.X,
                this.worldObject.GridPosition.Y + movement.Value.Y
            );
            gameController.backref.QueueForMovement(
                new MovementQueueItem(this.worldObject.GridPosition, newPosition, this.worldObject)
            );

            return new();
        }
    }
}
