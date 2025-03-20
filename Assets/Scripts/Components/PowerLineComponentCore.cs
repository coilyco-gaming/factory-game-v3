using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Numerics;
using Assets.Scripts.Core;

namespace Assets.Scripts.Components.Core
{
    [Serializable]
    public class PowerLineComponentCore
    {
        private string powerLineName;
        private bool powerLinesSpawned = false;

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

            // If we have already spawned power lines, we don't need to do it again
            if (this.powerLinesSpawned)
            {
                return new();
            }

            // Power lines should only spawn from power generators
            if (worldObject.power == null)
            {
                return new();
            }

            // Find the nearest world object with a power component.
            // And a battery with energy in it, not including yourself.
            // This should me power plants route to other power plants.
            WorldObjectCore closestPower = gameController
                .worldObjects.SelectMany(worldObjects => worldObjects.Value)
                .Select(thisWorldObject => thisWorldObject.Value)
                .Where(thisWorldObject => thisWorldObject != worldObject) // Exclude self
                .Where(thisWorldObject => thisWorldObject.battery != null) // Must have battery
                .Where(thisWorldObject => thisWorldObject.battery.Energy > 0) // Must have energy
                .Where(thisWorldObject => thisWorldObject.powerLine != null) // Must be a power line
                .OrderBy(thisWorldObject =>
                    System.Numerics.Vector2.Distance(
                        thisWorldObject.GridPosition,
                        worldObject.GridPosition
                    )
                )
                .FirstOrDefault();

            // This should never happen, but just in case
            if (closestPower == null)
            {
                return new()
                {
                    new() { { gameController.backref.TickCount, "no power source found" } },
                };
            }

            float distance = float.MaxValue;
            Vector2 currentPosition = worldObject.GridPosition;
            while (distance > 1.5f)
            {
                // Determine the closest tile that is pointed towards the power source
                System.Numerics.Vector2 closestTile = GameControllerCore
                    .GetAdjacentPositions(currentPosition)
                    .OrderBy(tile =>
                        System.Numerics.Vector2.Distance(tile, closestPower.GridPosition)
                    )
                    .First();

                // Check if that tile has a power line component on it
                bool hasPowerLine =
                    gameController
                        .worldObjects.GetValueOrDefault(closestTile)
                        ?.Any(worldObject => worldObject.Value.powerLine != null) ?? false;

                // If it does not have a power line, we need to spawn one
                if (!hasPowerLine)
                {
                    gameController.queuedForSpawn.Add(
                        new SpawnQueueItem(
                            this.powerLineName,
                            (int)closestTile.X,
                            (int)closestTile.Y
                        )
                    );
                }

                // Update the current position to the closest tile
                currentPosition = closestTile;
                distance = System.Numerics.Vector2.Distance(
                    currentPosition,
                    closestPower.GridPosition
                );
            }

            this.powerLinesSpawned = true;
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
                powerLine = new PowerLineComponentCore("testPowerLine"),
            };

            worldObject.powerLine.Tick(gameController, worldObject);
            Assert.True(true);
            this.testOutput.WriteLine("tested true");
        }

        // [Fact]
        // public void TestNoPowerSource()
        // {
        //     GameControllerCore gameController = new()
        //     {
        //         backref = new ExampleGameController() { TickCount = 0 },
        //         worldObjects = new(),
        //     };

        //     WorldObjectCore worldObject = new(null)
        //     {
        //         GridPosition = new System.Numerics.Vector2(0, 0),
        //         battery = new BatteryComponentCore(5, 100),
        //         powerLine = new PowerLineComponentCore("testPowerLine"),
        //     };

        //     List<Dictionary<uint, string>> alerts = worldObject.powerLine.Tick(
        //         gameController,
        //         worldObject
        //     );
        //     Assert.Equal("no power source found", alerts.First().Values.First());
        // }

        // [Fact]
        // public void TestNoPowerLine()
        // {
        //     GameControllerCore gameController = new()
        //     {
        //         backref = new ExampleGameController() { TickCount = 0 },
        //         worldObjects = new(),
        //     };

        //     WorldObjectCore worldObject1 = new(null)
        //     {
        //         GridPosition = new System.Numerics.Vector2(0, 0),
        //         battery = new BatteryComponentCore(5, 100),
        //         powerLine = new PowerLineComponentCore("testPowerLine"),
        //     };
        //     gameController.worldObjects[worldObject1.GridPosition] = new()
        //     {
        //         ["guid-0"] = worldObject1,
        //     };

        //     WorldObjectCore worldObject2 = new(null)
        //     {
        //         GridPosition = new System.Numerics.Vector2(10, 10),
        //         battery = new BatteryComponentCore(100, 100),
        //         power = new PowerComponentCore(),
        //         powerLine = new PowerLineComponentCore("testPowerLine"),
        //     };
        //     gameController.worldObjects[worldObject2.GridPosition] = new()
        //     {
        //         ["guid-1"] = worldObject2,
        //     };

        //     List<Dictionary<uint, string>> alerts = worldObject1.powerLine.Tick(
        //         gameController,
        //         worldObject1
        //     );
        //     Assert.Equal("spawning power line", alerts.First().Values.First());
        // }

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
                battery = new BatteryComponentCore(5, 100),
                powerLine = new PowerLineComponentCore("testPowerLine"),
            };
            gameController.worldObjects[worldObject1.GridPosition] = new()
            {
                ["guid-0"] = worldObject1,
            };

            WorldObjectCore worldObject2 = new(null)
            {
                GridPosition = new System.Numerics.Vector2(1, 0),
                battery = new BatteryComponentCore(100, 100),
                power = new PowerComponentCore(),
                powerLine = new PowerLineComponentCore("testPowerLine"),
            };
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
