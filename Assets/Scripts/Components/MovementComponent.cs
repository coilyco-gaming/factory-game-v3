namespace Assets.Scripts.Components.Core
{
    using System.Collections.Generic;
    using System.Numerics;
    using Roy_T.AStar.Graphs;
    using Roy_T.AStar.Paths;
    using Roy_T.AStar.Primitives;

    public class MovementComponentCore
    {
        public void Instantiate() { }

        public Vector2 DiamondSpiralPattern(Vector2 origin, Vector2 currentTarget, Vector2 mapSize)
        {
            // TODO: handles cases where you overrun the map

            Vector2 changeVector = currentTarget - origin;

            if (changeVector.X == 0 && changeVector.Y == 0)
            {
                currentTarget.Y += 1;
                return currentTarget;
            }

            if (changeVector.X >= 0 && changeVector.Y > 0)
            {
                currentTarget.X += 1;
                currentTarget.Y -= 1;
                return currentTarget;
            }

            if (changeVector.X > 0 && changeVector.Y <= 0)
            {
                currentTarget.X -= 1;
                currentTarget.Y -= 1;
                return currentTarget;
            }

            if (changeVector.X <= 0 && changeVector.Y < 0)
            {
                currentTarget.X -= 1;
                currentTarget.Y += 1;
                return currentTarget;
            }

            if (changeVector.X < 0 && changeVector.Y >= 0)
            {
                currentTarget.X += 1;
                currentTarget.Y += 1;
                // If the change vector would return you to 0,N
                // then add +1 to the Y
                if (currentTarget.X == origin.X)
                {
                    currentTarget.Y += 1;
                }
                return currentTarget;
            }

            return currentTarget;
        }

        public Vector2? GetMovement(
            Vector2 start,
            Vector2 end,
            Vector2 mapSize,
            List<Vector2> obstacles
        )
        {
            Vector2 movement;

            // Setup plain grid
            GridSize gridSize = new((int)mapSize.X, (int)mapSize.Y);
            Roy_T.AStar.Grids.Grid grid =
                Roy_T.AStar.Grids.Grid.CreateGridWithLateralAndDiagonalConnections(
                    gridSize,
                    new Size(Distance.FromMeters(1), Distance.FromMeters(1)),
                    Velocity.FromMetersPerSecond(1)
                );

            foreach (Vector2 obstacle in obstacles)
            {
                // Nothing here
                if (obstacle == null)
                {
                    continue;
                }
                // Dont block on yourself
                if (obstacle.X == start.X && obstacle.Y == start.Y)
                {
                    continue;
                }
                // Dont block on the target
                if (obstacle.X == end.X && obstacle.Y == end.Y)
                {
                    continue;
                }
                // Register obstacles
                grid.DisconnectNode(new GridPosition((int)obstacle.X, (int)obstacle.Y));
            }

            // Find the path
            PathFinder pathFinder = new();
            Path path = pathFinder.FindPath(
                new GridPosition((int)start.X, (int)start.Y),
                new GridPosition((int)end.X, (int)end.Y),
                grid
            );

            // No path found
            if (path == null || path.Edges.Count == 0)
            {
                return null;
            }

            // Derive the movement vector from the next node on the path
            IEdge edge = path.Edges[0];
            Vector2 nextPosition = new(edge.End.Position.X, edge.End.Position.Y);
            movement = nextPosition - start;

            return movement;
        }
    }
}

#if UNITY_6000
namespace Assets.Scripts.Components.Unity
{
    using System.Collections.Generic;
    using Assets.Scripts.Components.Core;
    using UnityEngine;

    public class MovementComponent : MonoBehaviour
    {
        public readonly MovementComponentCore core = new();

        public void Instantiate() => this.core.Instantiate();

        public System.Numerics.Vector2 DiamondSpiralPattern(
            System.Numerics.Vector2 origin,
            System.Numerics.Vector2 currentTarget
        ) => this.core.DiamondSpiralPattern(origin, currentTarget);

        public System.Numerics.Vector2? GetMovement(
            System.Numerics.Vector2 start,
            System.Numerics.Vector2 end,
            System.Numerics.Vector2 mapSize,
            System.Numerics.Vector2[] obstacles
        ) =>
            this.core.GetMovement(
                start,
                end,
                mapSize,
                new List<System.Numerics.Vector2>(obstacles)
            );
    }
}
#endif

namespace Assets.Scripts.Components.Tests
{
    using Assets.Scripts.Components.Core;
    using Xunit;

    public class MovementComponentTest
    {
        [Fact]
        public void TestTrue()
        {
            MovementComponentCore movement = new();
            movement.Instantiate();
            Assert.True(true);
        }
    }
}
