using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using Assets.Scripts.Core;

namespace Assets.Scripts.Components.Core
{
    [Serializable]
    public class PowerLineComponentCore
    {
        private string powerLineName;
        private uint powerLineSpawn = 10;
        private float powerLineSpawnPercent = 0.1f;
        private uint powerLineSpawnCost = 1;

        public PowerLineComponentCore(string powerLineName = "")
        {
            this.powerLineName = powerLineName;
        }

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
            activity.SetParentId(gameController.backref.WorldObjectTickActivity.Id);

            // Only generate power lines when the battery is nearly empty
            if (worldObject.battery.PercentEnergy < this.powerLineSpawnPercent)
            {
                return new();
            }

            // If you are here, you want to spawn a power line, but can't afford it.
            if (worldObject.battery.Energy < this.powerLineSpawnCost)
            {
                return new List<Dictionary<uint, string>>
                {
                    new()
                    {
                        {
                            gameController.backref.TickCount,
                            "not enough energy to spawn power line"
                        },
                    },
                };
            }
            worldObject.battery.Energy -= this.powerLineSpawnCost;

            // TODO: more exit early conditions

            // Find the nearest world object with a power component
            WorldObjectCore closestPower = gameController
                .worldObjects.SelectMany(worldObjects => worldObjects.Value)
                .Where(worldObject => worldObject.Value?.power != null)
                .OrderBy(thisWorldObject =>
                    System.Numerics.Vector2.Distance(
                        thisWorldObject.Value.GridPosition,
                        worldObject.GridPosition
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
                .GetAdjacentPositions(worldObject.GridPosition)
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
                    new() { { gameController.backref.TickCount, "spawning power line" } },
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
                battery = new BatteryComponentCore(100, 100),
            };
            worldObject.powerLine = new PowerLineComponentCore("testPowerLine");

            worldObject.powerLine.Tick(gameController, worldObject);
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
                battery = new BatteryComponentCore(0, 100),
            };
            worldObject.powerLine = new PowerLineComponentCore("testPowerLine");

            List<Dictionary<uint, string>> alerts = worldObject.powerLine.Tick(
                gameController,
                worldObject
            );
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
                battery = new BatteryComponentCore(0, 100),
            };
            worldObject1.powerLine = new PowerLineComponentCore("testPowerLine");
            gameController.worldObjects[worldObject1.GridPosition] = new()
            {
                ["guid-0"] = worldObject1,
            };

            WorldObjectCore worldObject2 = new(null)
            {
                GridPosition = new System.Numerics.Vector2(10, 10),
            };
            worldObject2.battery = new BatteryComponentCore(100, 100);
            worldObject2.power = new PowerComponentCore();
            worldObject2.powerLine = new PowerLineComponentCore("testPowerLine");
            gameController.worldObjects[worldObject2.GridPosition] = new()
            {
                ["guid-1"] = worldObject2,
            };

            List<Dictionary<uint, string>> alerts = worldObject1.powerLine.Tick(
                gameController,
                worldObject1
            );
            Assert.Equal("spawning power line", alerts.First().Values.First());
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
                battery = new BatteryComponentCore(0, 100),
            };
            worldObject1.powerLine = new PowerLineComponentCore("testPowerLine");
            gameController.worldObjects[worldObject1.GridPosition] = new()
            {
                ["guid-0"] = worldObject1,
            };

            WorldObjectCore worldObject2 = new(null)
            {
                GridPosition = new System.Numerics.Vector2(1, 0),
            };
            worldObject2.battery = new BatteryComponentCore(100, 100);
            worldObject2.power = new PowerComponentCore();
            worldObject2.powerLine = new PowerLineComponentCore("testPowerLine");
            gameController.worldObjects[worldObject2.GridPosition] = new()
            {
                ["guid-1"] = worldObject2,
            };

            List<Dictionary<uint, string>> alerts = worldObject1.powerLine.Tick(
                gameController,
                worldObject1
            );
            Assert.Equal(0, alerts.Count);
        }
    }
}
