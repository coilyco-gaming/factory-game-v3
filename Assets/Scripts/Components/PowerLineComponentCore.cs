using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Numerics;
using Assets.Scripts.Core;
using UnityEngine;

namespace Assets.Scripts.Components.Core
{
    [Serializable]
    public class PowerLineComponentCore
    {
        private WorldObjectCore worldObject;
        private string powerLineName;

        public PowerLineComponentCore(WorldObjectCore worldObject, string powerLineName)
        {
            this.worldObject = worldObject;
            this.powerLineName = powerLineName;
        }

        public List<Dictionary<uint, string>> Tick(GameControllerCore gameController)
        {
            using Activity activity = gameController.backref.ActivitySource.StartActivity(
                this.GetType().Name
            );
            activity.SetTag("WorldObjectType", this.worldObject.worldObjectType);

            // TODO: some kind of early exit condition...
            // TODO: because we don't want to query the world objects every tick

            // Find the nearest world object with a power component
            WorldObjectCore closestPower = gameController
                .worldObjects.SelectMany(worldObjects => worldObjects.Value)
                .Where(worldObject => worldObject.Value?.power != null)
                .OrderBy(worldObject =>
                    System.Numerics.Vector2.Distance(
                        worldObject.Value.GridPosition,
                        this.worldObject.GridPosition
                    )
                )
                .Select(worldObject => worldObject.Value)
                .FirstOrDefault();

            // This should never happen, but just in case
            if (closestPower == null)
            {
                return new List<Dictionary<uint, string>>
                {
                    new() { { gameController.backref.TickCount, "no power source found" } },
                };
            }

            // Determine the closest tile that is pointed towards the power source
            System.Numerics.Vector2 closestTile = GameControllerCore
                .GetAdjacentPositions(this.worldObject.GridPosition)
                .OrderBy(tile => System.Numerics.Vector2.Distance(tile, closestPower.GridPosition))
                .First();

            // Check if that tile has a power line component on it
            bool hasPowerLine =
                gameController
                    .worldObjects.GetValueOrDefault(closestTile)
                    ?.Any(worldObject => worldObject.Value.powerLine != null) ?? false;

            // Alert if the tile does not have a power line component
            if (!hasPowerLine)
            {
                gameController.queuedForSpawn.Add(
                    new SpawnQueueItem(this.powerLineName, (int)closestTile.X, (int)closestTile.Y)
                );

                return new List<Dictionary<uint, string>>
                {
                    new() { { gameController.backref.TickCount, "no power line found on tile" } },
                };
            }

            return new();
        }
    }
}

namespace Assets.Scripts.Components.Tests
{
    using Assets.Scripts.Components.Core;
    using Xunit;
    using Xunit.Abstractions;

    public class PowerLineComponentCoreTest
    {
        private ITestOutputHelper testOutput;

        public PowerLineComponentCoreTest(ITestOutputHelper testOutputHelper)
        {
            this.testOutput = testOutputHelper;
        }

        [Fact]
        public void TestTrue()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController() { TickCount = 0 },
                worldObjects = new(),
            };

            WorldObjectCore worldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(0, 0),
            };
            worldObject.powerLine = new PowerLineComponentCore(worldObject, "testPowerLine");

            worldObject.powerLine.Tick(gameController);
            Assert.True(true);
            this.testOutput.WriteLine("tested true");
        }

        [Fact]
        public void TestNoPowerSource()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController() { TickCount = 0 },
                worldObjects = new(),
            };

            WorldObjectCore worldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(0, 0),
            };
            worldObject.powerLine = new PowerLineComponentCore(worldObject, "testPowerLine");

            List<Dictionary<uint, string>> alerts = worldObject.powerLine.Tick(gameController);
            Assert.Equal("no power source found", alerts.First().Values.First());
        }

        [Fact]
        public void TestNoPowerLine()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController() { TickCount = 0 },
                worldObjects = new(),
            };

            WorldObjectCore worldObject1 = new(null)
            {
                GridPosition = new System.Numerics.Vector2(0, 0),
            };
            worldObject1.powerLine = new PowerLineComponentCore(worldObject1, "testPowerLine");
            gameController.worldObjects[worldObject1.GridPosition] = new()
            {
                ["guid-0"] = worldObject1,
            };

            WorldObjectCore worldObject2 = new(null)
            {
                GridPosition = new System.Numerics.Vector2(10, 10),
            };
            worldObject2.battery = new BatteryComponentCore(worldObject2, 100, 100);
            worldObject2.power = new PowerComponentCore(
                worldObject2,
                worldObject2.battery,
                new ResourcesComponentCore(null, 100, 100)
            );
            worldObject2.powerLine = new PowerLineComponentCore(worldObject2, "testPowerLine");
            gameController.worldObjects[worldObject2.GridPosition] = new()
            {
                ["guid-1"] = worldObject2,
            };

            List<Dictionary<uint, string>> alerts = worldObject1.powerLine.Tick(gameController);
            Assert.Equal("no power line found on tile", alerts.First().Values.First());
        }

        [Fact]
        public void TestSuccess()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController() { TickCount = 0 },
                worldObjects = new(),
            };

            WorldObjectCore worldObject1 = new(null)
            {
                GridPosition = new System.Numerics.Vector2(0, 0),
            };
            worldObject1.powerLine = new PowerLineComponentCore(worldObject1, "testPowerLine");
            gameController.worldObjects[worldObject1.GridPosition] = new()
            {
                ["guid-0"] = worldObject1,
            };

            WorldObjectCore worldObject2 = new(null)
            {
                GridPosition = new System.Numerics.Vector2(1, 0),
            };
            worldObject2.battery = new BatteryComponentCore(worldObject2, 100, 100);
            worldObject2.power = new PowerComponentCore(
                worldObject2,
                worldObject2.battery,
                new ResourcesComponentCore(null, 100, 100)
            );
            worldObject2.powerLine = new PowerLineComponentCore(worldObject2, "testPowerLine");
            gameController.worldObjects[worldObject2.GridPosition] = new()
            {
                ["guid-1"] = worldObject2,
            };

            List<Dictionary<uint, string>> alerts = worldObject1.powerLine.Tick(gameController);
            Assert.Equal(0, alerts.Count);
        }
    }
}
