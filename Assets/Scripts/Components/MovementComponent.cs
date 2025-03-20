namespace Assets.Scripts.Components.Core
{
    using System;
    using System.Collections.Generic;
    using System.Diagnostics;
    using System.Linq;
    using Assets.Scripts.Core;
    using EpPathFinding.cs;

    [Serializable]
    public class MovementComponentCore
    {
        private List<System.Numerics.Vector2> path = new();
        private int pathIndex = 0;

        public List<Dictionary<uint, string>> Tick(
            GameControllerCore gameController,
            WorldObjectCore worldObject
        )
        {
            using Activity activity = gameController.backref.ActivitySource.StartActivity(
                this.GetType().Name
            );
            activity.SetTag("WorldObjectType", worldObject.worldObjectType);
            activity.SetTag("tick", gameController.backref.TickCount);

            if (
                worldObject.dispatchReceivers.Count != 0
                && worldObject.dispatchReceivers.First().targetPosition == null
            )
            {
                // We have no target position
                return new()
                {
                    new() { { gameController.backref.TickCount, "no target position" } },
                };
            }

            System.Numerics.Vector2 start = worldObject.GridPosition;
            System.Numerics.Vector2 end = worldObject
                .dispatchReceivers.First()
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

            System.Numerics.Vector2 nextPosition;

            bool obstacleInWay = false;

            if (this.path.Count == 0)
            {
                // If there is an obstacle in the way, we need to recalculate the path
                obstacleInWay = gameController
                    .worldObjects[new System.Numerics.Vector2((int)end.X, (int)end.Y)]
                    .Where(worldObject =>
                        !worldObject.Value.mobile || !worldObject.Value.passThrough
                    )
                    .Any();
            }

            if (obstacleInWay || this.path.Count == 0)
            {
                // Get a movement vector to the next position on our path
                List<System.Numerics.Vector2> path = PathfindingComponentCore.GetPosition(
                    start: start,
                    end: end,
                    grid: gameController.backref.Map.Grid.Clone() as StaticGrid
                );

                if (path == null || path.Count == 0)
                {
                    // No available path
                    return new()
                    {
                        new() { { gameController.backref.TickCount, "no available path" } },
                    };
                }
                this.path = path;
                this.pathIndex = 0; // 0 is our current position
            }

            // Get the next position on our path by indexing into the path lsit
            this.pathIndex++;
            nextPosition = this.path[this.pathIndex];

            // Queue up movement
            gameController.backref.QueueForMovement(
                new MovementQueueItem(worldObject.GridPosition, nextPosition, worldObject)
            );

            return new();
        }
    }
}
