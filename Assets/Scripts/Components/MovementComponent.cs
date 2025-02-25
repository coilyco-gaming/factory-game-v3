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

        public Vector2? DiamondSpiralPattern(
            Vector2 origin,
            Vector2 currentTarget,
            Vector2 mapSize,
            int depth = 0
        )
        {
            Vector2 changeVector = currentTarget - origin;

            // We've check all the way around the map
            if (depth > mapSize.X + mapSize.Y)
            {
                return null;
            }

            // case 1a: (1, 1) -> (1, 2): upwards, this happens exactly once
            if (changeVector.X == 0 && changeVector.Y == 0)
            {
                currentTarget.Y += 1;
            }

            // case 2a: (1, 2) -> (2, 1): towards bottom right
            if (changeVector.X >= 0 && changeVector.Y > 0)
            {
                currentTarget.X += 1;
                currentTarget.Y -= 1;
            }

            // case 3a: (2, 1) -> (1, 0): towards bottom left
            if (changeVector.X > 0 && changeVector.Y <= 0)
            {
                currentTarget.X -= 1;
                currentTarget.Y -= 1;
            }

            // case 4a: (1, 0) -> (0, 1): towards top left
            if (changeVector.X <= 0 && changeVector.Y < 0)
            {
                currentTarget.X -= 1;
                currentTarget.Y += 1;
            }

            // case 5a: (0, 2) -> (1, 3):
            //   needs to handle an origin that is farther away
            //   than our simple (1,1) origin
            //   so we an example with a (2,2) origin instead
            if (changeVector.X < 0 && changeVector.Y >= 0)
            {
                currentTarget.X += 1;
                currentTarget.Y += 1;
                // case 5b: (0, 1) -> (1, 3)
                if (currentTarget.X == origin.X)
                {
                    // If the change vector would return you to 0,N
                    // then add +1 to the Y
                    currentTarget.Y += 1;
                }
            }

            if (
                currentTarget.X < 0
                || currentTarget.Y < 0
                || currentTarget.X > mapSize.X
                || currentTarget.Y > mapSize.Y
            )
            {
                // If the target is out of bounds, recurse to find a new target
                return this.DiamondSpiralPattern(origin, currentTarget, mapSize, depth + 1);
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

        public System.Numerics.Vector2? DiamondSpiralPattern(
            System.Numerics.Vector2 origin,
            System.Numerics.Vector2 currentTarget,
            System.Numerics.Vector2 mapSize
        ) => this.core.DiamondSpiralPattern(origin, currentTarget, mapSize);

        public System.Numerics.Vector2? GetMovement(
            System.Numerics.Vector2 start,
            System.Numerics.Vector2 end,
            System.Numerics.Vector2 mapSize,
            List<System.Numerics.Vector2> obstacles
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

        [Fact]
        public void TestDiamondSpiralPatternCase1a()
        {
            // case 1a: (1, 1) -> (1, 2)
            MovementComponentCore movement = new();
            movement.Instantiate();
            System.Numerics.Vector2 origin = new(1, 1);
            System.Numerics.Vector2 mapSize = new(10, 10);
            System.Numerics.Vector2 currentTarget = new(1, 1);
            System.Numerics.Vector2 expected = new(1, 2);
            System.Numerics.Vector2? actual = movement.DiamondSpiralPattern(
                origin,
                currentTarget,
                mapSize
            );
            Assert.Equal(expected, actual);
        }

        [Fact]
        public void TestDiamondSpiralPatternCase1b()
        {
            // case 1b:
            //   if we are at the top of the map
            //   then apply case 2a style movement
            MovementComponentCore movement = new();
            movement.Instantiate();
            System.Numerics.Vector2 origin = new(1, 10);
            System.Numerics.Vector2 mapSize = new(10, 10);
            System.Numerics.Vector2 currentTarget = new(1, 10);
            System.Numerics.Vector2 expected = new(2, 10);
            System.Numerics.Vector2? actual = movement.DiamondSpiralPattern(
                origin,
                currentTarget,
                mapSize
            );
            Assert.Equal(expected, actual);
        }

        [Fact]
        public void TestDiamondSpiralPatternCase1c()
        {
            // case 1c:
            //   if we are at the top right corner
            //   then apply case 3a style movement
            MovementComponentCore movement = new();
            movement.Instantiate();
            System.Numerics.Vector2 origin = new(10, 10);
            System.Numerics.Vector2 mapSize = new(10, 10);
            System.Numerics.Vector2 currentTarget = new(10, 10);
            System.Numerics.Vector2 expected = new(10, 9);
            System.Numerics.Vector2? actual = movement.DiamondSpiralPattern(
                origin,
                currentTarget,
                mapSize
            );
            Assert.Equal(expected, actual);
        }

        [Fact]
        public void TestDiamondSpiralPatternCase2a()
        {
            // case 2a: (1, 2) -> (2, 1)
            MovementComponentCore movement = new();
            movement.Instantiate();
            System.Numerics.Vector2 origin = new(1, 1);
            System.Numerics.Vector2 mapSize = new(10, 10);
            System.Numerics.Vector2 currentTarget = new(1, 2);
            System.Numerics.Vector2 expected = new(2, 1);
            System.Numerics.Vector2? actual = movement.DiamondSpiralPattern(
                origin,
                currentTarget,
                mapSize
            );
            Assert.Equal(expected, actual);
        }

        [Fact]
        public void TestDiamondSpiralPatternCase2b()
        {
            MovementComponentCore movement = new();
            movement.Instantiate();
            System.Numerics.Vector2 origin = new(1, 1);
            System.Numerics.Vector2 mapSize = new(10, 10);
            System.Numerics.Vector2 currentTarget = new(1, 2);
            System.Numerics.Vector2 expected = new(2, 1);
            System.Numerics.Vector2? actual = movement.DiamondSpiralPattern(
                origin,
                currentTarget,
                mapSize
            );
            Assert.Equal(expected, actual);
        }

        [Fact]
        public void TestDiamondSpiralPatternCase3a()
        {
            // case 3a: (2, 1) -> (1, 0)
            MovementComponentCore movement = new();
            movement.Instantiate();
            System.Numerics.Vector2 origin = new(1, 1);
            System.Numerics.Vector2 mapSize = new(10, 10);
            System.Numerics.Vector2 currentTarget = new(2, 1);
            System.Numerics.Vector2 expected = new(1, 0);
            System.Numerics.Vector2? actual = movement.DiamondSpiralPattern(
                origin,
                currentTarget,
                mapSize
            );
            Assert.Equal(expected, actual);
        }

        [Fact]
        public void TestDiamondSpiralPatternCase4a()
        {
            // case 4a: (1, 0) -> (0, 1)
            MovementComponentCore movement = new();
            movement.Instantiate();
            System.Numerics.Vector2 origin = new(1, 1);
            System.Numerics.Vector2 mapSize = new(10, 10);
            System.Numerics.Vector2 currentTarget = new(1, 0);
            System.Numerics.Vector2 expected = new(0, 1);
            System.Numerics.Vector2? actual = movement.DiamondSpiralPattern(
                origin,
                currentTarget,
                mapSize
            );
            Assert.Equal(expected, actual);
        }

        [Fact]
        public void TestDiamondSpiralPatternCase5a()
        {
            // case 5a: (0, 2) -> (1, 3)
            //   needs to handle an origin that is farther away
            //   than our simple (1,1) origin
            //   so we an example with a (2,2) origin instead
            MovementComponentCore movement = new();
            movement.Instantiate();
            System.Numerics.Vector2 origin = new(2, 2);
            System.Numerics.Vector2 mapSize = new(10, 10);
            System.Numerics.Vector2 currentTarget = new(0, 2);
            System.Numerics.Vector2 expected = new(1, 3);
            System.Numerics.Vector2? actual = movement.DiamondSpiralPattern(
                origin,
                currentTarget,
                mapSize
            );
            Assert.Equal(expected, actual);
        }

        [Fact]
        public void TestDiamondSpiralPatternCase5b()
        {
            // case 5b: (0, 1) -> (1, 3)
            MovementComponentCore movement = new();
            movement.Instantiate();
            System.Numerics.Vector2 origin = new(1, 1);
            System.Numerics.Vector2 mapSize = new(10, 10);
            System.Numerics.Vector2 currentTarget = new(0, 1);
            System.Numerics.Vector2 expected = new(1, 3);
            System.Numerics.Vector2? actual = movement.DiamondSpiralPattern(
                origin,
                currentTarget,
                mapSize
            );
            Assert.Equal(expected, actual);
        }

        [Fact]
        public void TestDiamondSpiralPatternCase6End()
        {
            MovementComponentCore movement = new();
            movement.Instantiate();
            System.Numerics.Vector2 origin = new(1, 1);
            System.Numerics.Vector2 mapSize = new(10, 10);
            System.Numerics.Vector2 currentTarget = new(10, 10);
            System.Numerics.Vector2? actual = movement.DiamondSpiralPattern(
                origin,
                currentTarget,
                mapSize
            );
            Assert.Equal(null, actual);
        }
    }
}
