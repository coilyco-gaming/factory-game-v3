using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using Assets.Scripts.Core;
using UnityEngine;

namespace Assets.Scripts.Components.Core
{
    [Serializable]
    public class MiningComponentCore
    {
        private int MiningSpeed { get; set; } = 1;
        private int MiningEnergyCost { get; set; } = 1;
        private string targetType;
        private GameContent.Item TargetItem =>
            this.gameContent.Items.GetValueOrDefault(this.targetType);

        private GameContent gameContent;

        public MiningComponentCore(
            GameContent gameContent,
            string targetType,
            int miningSpeed = 1,
            int miningEneryCost = 1
        )
        {
            this.gameContent = gameContent;
            this.targetType = targetType;
            this.MiningSpeed = miningSpeed;
            this.MiningEnergyCost = miningEneryCost;
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

            // If the target can manifest, then just create the resource and return
            if (this.TargetItem.CreateFromNothing)
            {
                try
                {
                    worldObject.resources.CreateResources(this.targetType, (uint)this.MiningSpeed);
                }
                catch (ResourcesComponentCore.ResourceException) { }
                return new();
            }

            // Get objects on our tile
            List<WorldObjectCore> objectsOnTile = gameController
                .worldObjects.GetValueOrDefault(worldObject.gridPosition)
                ?.Values.ToList();

            // If there are no objects on our tile, or only one object (us), return
            if (objectsOnTile == null || objectsOnTile.Count == 1)
            {
                return new() { new() { { gameController.backref.TickCount, "nothing to mine" } } };
            }

            // Get ore producing world object on our tile
            WorldObjectCore oreObject = objectsOnTile
                .Where(worldObject => worldObject.worldObjectType == this.targetType)
                .FirstOrDefault();

            // If there are no ores on our tile, return
            if (oreObject == null)
            {
                return new()
                {
                    new()
                    {
                        { gameController.backref.TickCount, "no target object to mine on tile" },
                    },
                };
            }

            // Get the actual ore resources component
            ResourcesComponentCore oreResources = oreObject.resources;
            if (oreResources == null)
            {
                return new()
                {
                    new()
                    {
                        {
                            gameController.backref.TickCount,
                            "no resources to mine on target object"
                        },
                    },
                };
            }

            // Check if we have enough energy to mine
            if (worldObject.battery.Energy < this.MiningEnergyCost)
            {
                return new()
                {
                    new() { { gameController.backref.TickCount, "not enough energy to mine" } },
                };
            }

            // Consume energy
            worldObject.battery.Energy -= this.MiningEnergyCost;

            // Take "mining speed" worth of resources
            try
            {
                worldObject.resources.TakeResources(
                    oreResources,
                    this.targetType,
                    (uint)this.MiningSpeed
                );
            }
            catch (ResourcesComponentCore.ResourceException) { }

            return new();
        }
    }
}

namespace Assets.Scripts.Components.Tests
{
    using System.Numerics;
    using Assets.Scripts.Components.Core;
    using Xunit;
    using Xunit.Abstractions;

    internal class MiningGameContent : GameContent
    {
        public override Dictionary<string, Item> Items =>
            new()
            {
                { "Ore", new Item("Ore") }, //
                { "Stone", new Item("Ore", createFromNothing: true) }, //
            };
    }

    public class MiningComponentCoreTest
    {
        private ITestOutputHelper testOutput;

        public MiningComponentCoreTest(ITestOutputHelper testOutputHelper)
        {
            this.testOutput = testOutputHelper;
        }

        [Fact]
        public void TestTrue()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };
            WorldObjectCore worldObject = new(null)
            {
                resources = new ResourcesComponentCore(
                    new MiningGameContent(),
                    weightCapacity: uint.MaxValue,
                    volumeCapacity: uint.MaxValue
                ),
                battery = new BatteryComponentCore(startingEnergy: 1000),
            };
            MiningComponentCore mining = new(new MiningGameContent(), "Ore", 1, 1);
            mining.Tick(gameController, worldObject);
            Assert.True(true);
            this.testOutput.WriteLine("tested true");
        }

        [Fact]
        public void TestTakesResources()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };

            WorldObjectCore miningWorldObject = new(null)
            {
                resources = new ResourcesComponentCore(
                    new MiningGameContent(),
                    weightCapacity: uint.MaxValue,
                    volumeCapacity: uint.MaxValue
                ),
                gridPosition = new Vector2(0, 0),
                battery = new BatteryComponentCore(startingEnergy: 1000),
            };
            miningWorldObject.guid = miningWorldObject.CreateGuid();
            MiningComponentCore mining = new(new MiningGameContent(), "Ore", 1, 1);
            gameController.worldObjects[miningWorldObject.GridPosition] = new()
            {
                { miningWorldObject.guid, miningWorldObject },
            };

            WorldObjectCore oreWorldObject = new(null)
            {
                worldObjectType = "Ore",
                resources = new ResourcesComponentCore(
                    new MiningGameContent(),
                    weightCapacity: uint.MaxValue,
                    volumeCapacity: uint.MaxValue
                ),
                gridPosition = new Vector2(0, 0),
            };
            oreWorldObject.guid = oreWorldObject.CreateGuid();
            oreWorldObject.resources.CreateResources("Ore", 1000);
            gameController.worldObjects[oreWorldObject.gridPosition][oreWorldObject.guid] =
                oreWorldObject;

            mining.Tick(gameController, miningWorldObject);

            Assert.Equal(1u, miningWorldObject.resources.resources.GetValueOrDefault("Ore"));
        }

        [Fact]
        public void TestManifests()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };
            WorldObjectCore worldObject = new(null)
            {
                resources = new ResourcesComponentCore(
                    new MiningGameContent(),
                    weightCapacity: uint.MaxValue,
                    volumeCapacity: uint.MaxValue
                ),
                battery = new BatteryComponentCore(startingEnergy: 1000),
            };
            MiningComponentCore mining = new(new MiningGameContent(), "Stone", 1, 1);
            mining.Tick(gameController, worldObject);
            Assert.Equal(1u, worldObject.resources.resources.GetValueOrDefault("Stone"));
        }
    }
}
