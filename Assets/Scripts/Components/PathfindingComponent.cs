namespace Assets.Scripts.Components.Core
{
    using System;
    using System.Collections.Generic;
    using EpPathFinding.cs;
    using UnityEngine;

    public class PathfindingComponentCore
    {
        public static System.Numerics.Vector2? DiamondSpiralPattern(
            System.Numerics.Vector2 origin,
            System.Numerics.Vector2 currentTarget,
            System.Numerics.Vector2 mapSize,
            int depth = 0
        )
        {
            System.Numerics.Vector2 changeVector = currentTarget - origin;

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
                return PathfindingComponentCore.DiamondSpiralPattern(
                    origin,
                    currentTarget,
                    mapSize,
                    depth + 1
                );
            }

            return currentTarget;
        }

        public static System.Numerics.Vector2? GetPosition(
            System.Numerics.Vector2 start,
            System.Numerics.Vector2 end,
            StaticGrid grid
        )
        {
            System.Numerics.Vector2? position = null;
            GridPos startGridPosition = new((int)start.X, (int)start.Y);
            GridPos endGridPosition = new((int)end.X, (int)end.Y);

            JumpPointParam jpParam = new(
                grid,
                startGridPosition,
                endGridPosition,
                iAllowEndNodeUnWalkable: EndNodeUnWalkableTreatment.ALLOW,
                iDiagonalMovement: DiagonalMovement.IfAtLeastOneWalkable
            );

            // TODO: this only seems to find a valid path 1 ~ 2 times a tick
            List<GridPos> resultPathList = JumpPointFinder.FindPath(jpParam);

            Debug.Log($"Pathfinding from {start} to {end}: {resultPathList.Count}");

            if (resultPathList != null && resultPathList.Count != 0)
            {
                // Derive the position vector from the next node on the path
                int nextX = resultPathList[1].x;
                int nextY = resultPathList[1].y;

                // The pathfinding algo is coded to "jump" when possible
                // so we need to clip the movement if the next node is more than 1 away
                nextX = Math.Clamp(nextX, (int)start.X - 1, (int)start.X + 1);
                nextY = Math.Clamp(nextY, (int)start.Y - 1, (int)start.Y + 1);

                System.Numerics.Vector2 nextPosition = new(nextX, nextY);
                position = nextPosition - start;
            }

            return position;
        }

        public static System.Numerics.Quaternion FacePosition(System.Numerics.Vector2 position)
        {
            double radians = Math.Atan2(position.X, position.Y);
            System.Numerics.Quaternion rotation = System.Numerics.Quaternion.CreateFromYawPitchRoll(
                0,
                0,
                (float)radians
            );
            return rotation;
        }

        public static System.Numerics.Quaternion FaceLocation(
            System.Numerics.Vector2 origin,
            System.Numerics.Vector2 target
        )
        {
            float yOffset = target.Y - origin.Y;
            float xOffset = target.X - origin.X;
            System.Numerics.Vector2 diff = new(xOffset, yOffset);
            return PathfindingComponentCore.FacePosition(diff);
        }
    }
}

namespace Assets.Scripts.Components.Tests
{
    using System;
    using System.Numerics;
    using Assets.Scripts.Components.Core;
    using Xunit;

    public class PathfindingComponentTest
    {
        [Fact]
        public void TestDiamondSpiralPatternCase1a()
        {
            // case 1a: (1, 1) -> (1, 2)
            System.Numerics.Vector2 origin = new(1, 1);
            System.Numerics.Vector2 mapSize = new(10, 10);
            System.Numerics.Vector2 currentTarget = new(1, 1);
            System.Numerics.Vector2 expected = new(1, 2);
            System.Numerics.Vector2? actual = PathfindingComponentCore.DiamondSpiralPattern(
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
            //   then apply case 2a style position
            System.Numerics.Vector2 origin = new(1, 10);
            System.Numerics.Vector2 mapSize = new(10, 10);
            System.Numerics.Vector2 currentTarget = new(1, 10);
            System.Numerics.Vector2 expected = new(2, 10);
            System.Numerics.Vector2? actual = PathfindingComponentCore.DiamondSpiralPattern(
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
            //   then apply case 3a style position
            System.Numerics.Vector2 origin = new(10, 10);
            System.Numerics.Vector2 mapSize = new(10, 10);
            System.Numerics.Vector2 currentTarget = new(10, 10);
            System.Numerics.Vector2 expected = new(10, 9);
            System.Numerics.Vector2? actual = PathfindingComponentCore.DiamondSpiralPattern(
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
            System.Numerics.Vector2 origin = new(1, 1);
            System.Numerics.Vector2 mapSize = new(10, 10);
            System.Numerics.Vector2 currentTarget = new(1, 2);
            System.Numerics.Vector2 expected = new(2, 1);
            System.Numerics.Vector2? actual = PathfindingComponentCore.DiamondSpiralPattern(
                origin,
                currentTarget,
                mapSize
            );
            Assert.Equal(expected, actual);
        }

        [Fact]
        public void TestDiamondSpiralPatternCase2b()
        {
            System.Numerics.Vector2 origin = new(1, 1);
            System.Numerics.Vector2 mapSize = new(10, 10);
            System.Numerics.Vector2 currentTarget = new(1, 2);
            System.Numerics.Vector2 expected = new(2, 1);
            System.Numerics.Vector2? actual = PathfindingComponentCore.DiamondSpiralPattern(
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
            System.Numerics.Vector2 origin = new(1, 1);
            System.Numerics.Vector2 mapSize = new(10, 10);
            System.Numerics.Vector2 currentTarget = new(2, 1);
            System.Numerics.Vector2 expected = new(1, 0);
            System.Numerics.Vector2? actual = PathfindingComponentCore.DiamondSpiralPattern(
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
            System.Numerics.Vector2 origin = new(1, 1);
            System.Numerics.Vector2 mapSize = new(10, 10);
            System.Numerics.Vector2 currentTarget = new(1, 0);
            System.Numerics.Vector2 expected = new(0, 1);
            System.Numerics.Vector2? actual = PathfindingComponentCore.DiamondSpiralPattern(
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
            System.Numerics.Vector2 origin = new(2, 2);
            System.Numerics.Vector2 mapSize = new(10, 10);
            System.Numerics.Vector2 currentTarget = new(0, 2);
            System.Numerics.Vector2 expected = new(1, 3);
            System.Numerics.Vector2? actual = PathfindingComponentCore.DiamondSpiralPattern(
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
            System.Numerics.Vector2 origin = new(1, 1);
            System.Numerics.Vector2 mapSize = new(10, 10);
            System.Numerics.Vector2 currentTarget = new(0, 1);
            System.Numerics.Vector2 expected = new(1, 3);
            System.Numerics.Vector2? actual = PathfindingComponentCore.DiamondSpiralPattern(
                origin,
                currentTarget,
                mapSize
            );
            Assert.Equal(expected, actual);
        }

        [Fact]
        public void TestDiamondSpiralPatternCase6End()
        {
            System.Numerics.Vector2 origin = new(1, 1);
            System.Numerics.Vector2 mapSize = new(10, 10);
            System.Numerics.Vector2 currentTarget = new(10, 10);
            System.Numerics.Vector2? actual = PathfindingComponentCore.DiamondSpiralPattern(
                origin,
                currentTarget,
                mapSize
            );
            Assert.Equal(null, actual);
        }

        [Fact]
        public void TestFacePosition1()
        {
            System.Numerics.Vector2 positionVector = new(0, 0);
            Quaternion actual = PathfindingComponentCore.FacePosition(positionVector);
            Quaternion expected = Quaternion.CreateFromYawPitchRoll(0, 0, 0);
            Assert.Equal(expected, actual);
        }

        [Fact]
        public void TestFacePosition2()
        {
            System.Numerics.Vector2 positionVector = new(0, 1);
            Quaternion actual = PathfindingComponentCore.FacePosition(positionVector);
            Quaternion expected = Quaternion.CreateFromYawPitchRoll(0, 0, 0);
            Assert.Equal(expected, actual);
        }

        [Fact]
        public void TestFacePosition3()
        {
            System.Numerics.Vector2 positionVector = new(1, 1);
            Quaternion actual = PathfindingComponentCore.FacePosition(positionVector);
            Quaternion expected = Quaternion.CreateFromYawPitchRoll(0, 0, (float)(1 * Math.PI / 4));
            Assert.Equal(expected, actual);
        }

        [Fact]
        public void TestFacePosition4()
        {
            System.Numerics.Vector2 positionVector = new(1, 0);
            Quaternion actual = PathfindingComponentCore.FacePosition(positionVector);
            Quaternion expected = Quaternion.CreateFromYawPitchRoll(0, 0, (float)(2 * Math.PI / 4));
            Assert.Equal(expected, actual);
        }

        [Fact]
        public void TestFacePosition5()
        {
            System.Numerics.Vector2 positionVector = new(1, -1);
            Quaternion actual = PathfindingComponentCore.FacePosition(positionVector);
            Quaternion expected = Quaternion.CreateFromYawPitchRoll(0, 0, (float)(3 * Math.PI / 4));
            Assert.Equal(expected, actual);
        }

        [Fact]
        public void TestFacePosition6()
        {
            System.Numerics.Vector2 positionVector = new(0, -1);
            Quaternion actual = PathfindingComponentCore.FacePosition(positionVector);
            Quaternion expected = Quaternion.CreateFromYawPitchRoll(0, 0, (float)(4 * Math.PI / 4));
            Assert.Equal(expected, actual);
        }

        [Fact]
        public void TestFaceLocation()
        {
            System.Numerics.Vector2 origin = new(0, 0);
            System.Numerics.Vector2 target = new(1, 1);
            Quaternion actual = PathfindingComponentCore.FaceLocation(origin, target);
            Quaternion expected = Quaternion.CreateFromYawPitchRoll(0, 0, (float)(Math.PI / 4));
            Assert.Equal(expected, actual);
        }
    }
}
