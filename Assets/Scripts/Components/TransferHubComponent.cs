// A type of component that tries to balance all available
// resources between all connected entities.

using System.Collections.Generic;
using System.Linq;
using System.Numerics;
using Assets.Scripts.Core;
using Assets.Scripts.WorldObjects.Core;

namespace Assets.Scripts.Components.Core
{
    public class TransferHubComponent
    {
        private GameControllerCore gameController;
        private WorldObjectCore core;
        private BatteryComponentCore battery;

        public TransferHubComponent(
            GameControllerCore gameController,
            WorldObjectCore core,
            BatteryComponentCore battery
        )
        {
            this.core = core;
            this.gameController = gameController;
            this.battery = battery;
        }

        public void Balance()
        {
            List<WorldObjectCore> localWorldObjects = this
                .gameController.GetAdjacentWorldObjects(this.core.GridPosition)
                .Where(worldObject => worldObject.Resources != null)
                .ToList();

            List<ProductionComponentCore> localProductionComponents = localWorldObjects
                .Select(worldObject => worldObject.Production)
                .Where(production => production != null)
                .ToList();

            List<string> localIngredients = localProductionComponents
                .SelectMany(production => production.ProductItem.Ingredients.Keys)
                .Distinct()
                .ToList();

            Dictionary<string, uint> localIngredientsCounts = localIngredients
                .ToDictionary(
                    ingredient => ingredient,
                    ingredient =>
                        (uint)
                            localProductionComponents
                                .SelectMany(production => production.ProductItem.Ingredients)
                                .Sum(ingredient => ingredient.Value)
                )
                .Where(ingredient => ingredient.Value > 0)
                .ToDictionary(ingredient => ingredient.Key, ingredient => ingredient.Value);

            // For each resource, distribute it evenly.
            foreach (KeyValuePair<string, uint> resource in localIngredientsCounts)
            {
                // Only include in this loop the local world objects whose factory
                // has this resource as an product or ingredient.
                List<WorldObjectCore> resourceScopedWorldObjects = localWorldObjects
                    .Where(worldObject =>
                        worldObject.Production.ProductItem.Ingredients.ContainsKey(resource.Key)
                        || worldObject.Production.ProductItem.Name == resource.Key
                    )
                    .ToList();

                // First consume all the resources in every surrounding container.
                // Aggregating the total amount of resources consumed.
                uint totalAmount = 0;
                foreach (WorldObjectCore worldObject in resourceScopedWorldObjects)
                {
                    if (worldObject.Resources.Resources.ContainsKey(resource.Key))
                    {
                        totalAmount += worldObject.Resources.Resources[resource.Key];
                        worldObject.Resources.Resources[resource.Key] = 0;
                    }
                }

                // Then distribute the resources evenly.
                uint amountPerContainer = (uint)(
                    totalAmount / (float)resourceScopedWorldObjects.Count
                );
                foreach (WorldObjectCore worldObject in resourceScopedWorldObjects)
                {
                    worldObject.Resources.Resources[resource.Key] = amountPerContainer;
                }

                // If there is a remainder, distribute it evenly.
                uint remainder = totalAmount % (uint)resourceScopedWorldObjects.Count;
                for (int i = 0; i < remainder; i++)
                {
                    resourceScopedWorldObjects[i].Resources.Resources[resource.Key] += 1;
                }
                this.battery.Energy -= 1;
            }
        }
    }
}

namespace Assets.Scripts.Components.Tests
{
    using System.Collections.Generic;
    using Assets.Scripts.Components.Core;
    using Assets.Scripts.Core;
    using Assets.Scripts.WorldObjects.Core;
    using Xunit;

    internal class TestGameContent : GameContent
    {
        public override Dictionary<string, Item> Items { get; } =
            new()
            {
                { "wood", new Item("wood", stackSize: 100) },
                { "iron", new Item("iron", stackSize: 100) },
                {
                    "nails",
                    new Item(
                        "nails",
                        stackSize: 100,
                        ingredients: new Dictionary<string, uint> { { "iron", 1 } }
                    )
                },
                {
                    "frame",
                    new Item(
                        "frame",
                        stackSize: 10,
                        ingredients: new Dictionary<string, uint> { { "iron", 16 } }
                    )
                },
                {
                    "planks",
                    new Item(
                        "planks",
                        stackSize: 10,
                        ingredients: new Dictionary<string, uint> { { "wood", 5 }, { "nails", 5 } }
                    )
                },
                {
                    "wall",
                    new Item(
                        "wall",
                        stackSize: 10,
                        ingredients: new Dictionary<string, uint>
                        {
                            { "planks", 4 },
                            { "nails", 4 },
                            { "frame", 1 },
                        }
                    )
                },
            };
    }

    public class TransferHubComponentTests
    {
        private WorldObjectCore WorldObject(GameControllerCore gameController)
        {
            Vector2 gridPosition = new(0, 0);
            ResourcesComponentCore resources = new(new TestGameContent(), 10000, 10000);
            BatteryComponentCore battery = new(1000, 1000);
            ProductionComponentCore production = new(
                new TestGameContent(),
                resources,
                battery,
                new List<InserterComponentCore>()
                {
                    new(battery, resources, "wood", 1),
                    new(battery, resources, "nails", 1),
                },
                "planks"
            );
            WorldObjectCore core = new(null)
            {
                Battery = battery,
                Resources = resources,
                GridPosition = gridPosition,
                Production = production,
            };
            TransferHubComponent transfer = new(gameController, core, battery);
            core.TransferHub = transfer;
            core.Guid = core.CreateGuid();
            gameController.worldObjects ??= new();
            if (!gameController.worldObjects.ContainsKey(core.GridPosition))
            {
                gameController.worldObjects[core.GridPosition] = new();
            }
            gameController.worldObjects[core.GridPosition][core.Guid] = core;
            return core;
        }

        [Fact]
        public void TestOneOneTransferNOOP()
        {
            GameControllerCore gameController = new() { worldObjects = new() };
            WorldObjectCore core1 = this.WorldObject(gameController);
            WorldObjectCore core2 = this.WorldObject(gameController);

            core1.Resources.CreateResources("wood", 100);
            core2.Resources.CreateResources("wood", 100);

            core1.TransferHub.Balance();
            core2.TransferHub.Balance();

            Assert.Equal(100u, core1.Resources.Resources["wood"]);
            Assert.Equal(100u, core2.Resources.Resources["wood"]);
        }

        [Fact]
        public void TestOneZeroTransfer()
        {
            GameControllerCore gameController = new() { worldObjects = new() };
            WorldObjectCore core1 = this.WorldObject(gameController);
            WorldObjectCore core2 = this.WorldObject(gameController);

            core1.Resources.CreateResources("wood", 100);
            core2.Resources.CreateResources("wood", 0);

            core1.TransferHub.Balance();
            core2.TransferHub.Balance();

            Assert.Equal(50u, core1.Resources.Resources["wood"]);
            Assert.Equal(50u, core2.Resources.Resources["wood"]);
        }

        [Fact]
        public void TestTwoTwoTransfer()
        {
            GameControllerCore gameController = new() { worldObjects = new() };
            WorldObjectCore core1 = this.WorldObject(gameController);
            WorldObjectCore core2 = this.WorldObject(gameController);

            core1.Resources.CreateResources("wood", 100);
            core1.Resources.CreateResources("nails", 0);
            core2.Resources.CreateResources("wood", 0);
            core2.Resources.CreateResources("nails", 100);

            core1.TransferHub.Balance();
            core2.TransferHub.Balance();

            Assert.Equal(50u, core1.Resources.Resources["wood"]);
            Assert.Equal(50u, core1.Resources.Resources["nails"]);
            Assert.Equal(50u, core2.Resources.Resources["wood"]);
            Assert.Equal(50u, core2.Resources.Resources["nails"]);
        }

        [Fact]
        public void TestTruncation()
        {
            GameControllerCore gameController = new() { worldObjects = new() };
            WorldObjectCore core1 = this.WorldObject(gameController);
            WorldObjectCore core2 = this.WorldObject(gameController);
            WorldObjectCore core3 = this.WorldObject(gameController);

            core1.Resources.CreateResources("wood", 100);
            core2.Resources.CreateResources("wood", 0);
            core3.Resources.CreateResources("wood", 0);

            core1.TransferHub.Balance();
            core2.TransferHub.Balance();
            core3.TransferHub.Balance();

            Assert.Equal(
                new List<float>
                {
                    core1.Resources.Resources["wood"],
                    core2.Resources.Resources["wood"],
                    core3.Resources.Resources["wood"],
                }.Sum(),
                100u
            );
            Assert.Equal(34u, core1.Resources.Resources["wood"]);
            Assert.Equal(33u, core2.Resources.Resources["wood"]);
            Assert.Equal(33u, core3.Resources.Resources["wood"]);
        }
    }
}
