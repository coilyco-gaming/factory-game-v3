using System.Collections.Generic;
using System.Linq;
using Assets.Scripts.Core;

namespace Assets.Scripts.Components.Core
{
    public class MiningComponentCore
    {
        private int MiningSpeed { get; set; } = 1;
        private int MiningEnergyCost { get; set; } = 1;
        private string targetType;
        private GameContent.Item TargetItem =>
            this.gameContent.Items.GetValueOrDefault(this.targetType);
        private WorldObjectCore worldObject;
        private GameContent gameContent;

        public MiningComponentCore(
            WorldObjectCore worldObject,
            GameContent gameContent,
            string targetType,
            int miningSpeed = 1,
            int miningEneryCost = 1
        )
        {
            this.worldObject = worldObject;
            if (worldObject.resources == null)
            {
                throw new GameControllerCore.MisconfigurationException(
                    "Mining component requires a resources component on its parent world object"
                );
            }
            if (worldObject.battery == null)
            {
                throw new GameControllerCore.MisconfigurationException(
                    "Mining component requires a battery component on its parent world object"
                );
            }
            this.gameContent = gameContent;
            this.targetType = targetType;
            this.MiningSpeed = miningSpeed;
            this.MiningEnergyCost = miningEneryCost;
        }

        public List<Dictionary<uint, string>> Tick(GameControllerCore gameController)
        {
            // If the target can manifest, then just create the resource and return
            if (this.TargetItem.Manifests)
            {
                this.worldObject.resources.CreateResources(this.targetType, (uint)this.MiningSpeed);
                return new();
            }

            // Get objects on our tile
            List<WorldObjectCore> objectsOnTile = gameController
                .worldObjects.GetValueOrDefault(this.worldObject.gridPosition)
                ?.Values.ToList();

            // If there are no objects on our tile, or only one object (us), return
            if (objectsOnTile == null || objectsOnTile.Count < 2)
            {
                return new List<Dictionary<uint, string>>
                {
                    new()
                    {
                        { gameController.backref.TickCount, "less then 2 objects on tile to mine" },
                    },
                };
            }

            // Get ore producing world object on our tile
            WorldObjectCore oreObject = objectsOnTile
                .Where(worldObject => worldObject.worldObjectType == this.targetType)
                .FirstOrDefault();

            // If there are no ores on our tile, return
            if (oreObject == null)
            {
                return new List<Dictionary<uint, string>>
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
                return new List<Dictionary<uint, string>>
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

            // Check if ore has resources
            if (oreResources.resources.GetValueOrDefault(this.targetType) < this.MiningSpeed)
            {
                return new List<Dictionary<uint, string>>
                {
                    new() { { gameController.backref.TickCount, "not enough ore to mine" } },
                };
            }

            // Check if we have enough energy to mine
            if (this.worldObject.battery.Energy < this.MiningEnergyCost)
            {
                return new List<Dictionary<uint, string>>
                {
                    new() { { gameController.backref.TickCount, "not enough energy to mine" } },
                };
            }

            // Consume energy
            this.worldObject.battery.Energy -= this.MiningEnergyCost;

            // Take "mining speed" worth of resources
            try
            {
                this.worldObject.resources.TakeResources(
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
    using Assets.Scripts.Unity;
    using Xunit;
    using Xunit.Abstractions;

    internal class MiningGameContent : GameContent
    {
        public override Dictionary<string, Item> Items =>
            new()
            {
                { "Ore", new Item("Ore") }, //
                { "Stone", new Item("Ore", manifests: true) }, //
            };
    }

    internal class MiningGameController : IGameController
    {
        public uint TickCount { get; set; } = 0;
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
                backref = new MiningGameController() { TickCount = 0 },
                worldObjects = new Dictionary<Vector2, Dictionary<string, WorldObjectCore>>(),
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
            MiningComponentCore mining = new(worldObject, new MiningGameContent(), "Ore", 1);
            mining.Tick(gameController);
            Assert.True(true);
            this.testOutput.WriteLine("tested true");
        }

        [Fact]
        public void TestTakesResouces()
        {
            GameControllerCore gameController = new()
            {
                backref = new MiningGameController() { TickCount = 0 },
                worldObjects = new Dictionary<Vector2, Dictionary<string, WorldObjectCore>>(),
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
            MiningComponentCore mining = new(miningWorldObject, new MiningGameContent(), "Ore", 1);
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
            };
            oreWorldObject.guid = oreWorldObject.CreateGuid();
            oreWorldObject.resources.CreateResources("Ore", 1000);
            gameController.worldObjects[oreWorldObject.gridPosition][oreWorldObject.guid] =
                oreWorldObject;

            mining.Tick(gameController);

            Assert.Equal(miningWorldObject.resources.resources.GetValueOrDefault("Ore"), 1u);
        }

        [Fact]
        public void TestManifests()
        {
            GameControllerCore gameController = new()
            {
                backref = new MiningGameController() { TickCount = 0 },
                worldObjects = new Dictionary<Vector2, Dictionary<string, WorldObjectCore>>(),
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
            MiningComponentCore mining = new(worldObject, new MiningGameContent(), "Stone", 1);
            mining.Tick(gameController);
            Assert.Equal(worldObject.resources.resources.GetValueOrDefault("Stone"), 1u);
        }
    }
}
