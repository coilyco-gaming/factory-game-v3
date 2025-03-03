// A type of component that tries to balance all available
// resources between all connected entities.

using System;
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
            this.battery =
                battery
                ?? throw new GameControllerCore.MisconfigurationException(
                    "Transfer Hub component requires a battery component"
                );
        }

        public void Balance()
        {
            // Do nothing if we don't have enough energy.
            List<WorldObjectCore> worldObjects = this
                .gameController.GetAdjacentWorldObjects(this.core.GridPosition)
                .Where(worldObject => worldObject.resources != null)
                .ToList();

            List<ProductionComponentCore> localProductionComponents = worldObjects
                .Select(worldObject => worldObject.production)
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
                this.BalanceResource(resource, worldObjects);
            }
        }

        private void BalanceResource(
            KeyValuePair<string, uint> resource,
            List<WorldObjectCore> _worldObjects
        )
        {
            // Only include in this loop the local world objects whose factory
            // has this resource as an product or ingredient.
            List<WorldObjectCore> worldObjects = _worldObjects
                .Where(worldObject =>
                    (
                        worldObject != null
                        && worldObject.production != null
                        && worldObject.production.ProductItem != null
                        && worldObject.production.ProductItem.Ingredients != null
                        && worldObject.production.ProductItem.Ingredients.Count != 0
                        && worldObject.production.ProductItem.Ingredients.ContainsKey(resource.Key)
                    )
                    || (
                        worldObject != null
                        && worldObject.production != null
                        && worldObject.production.ProductItem != null
                        && worldObject.production.ProductItem.Name == resource.Key
                    )
                )
                .ToList();

            // Also include yourself, but only if that doesn't duplicate.
            worldObjects.Add(this.core);
            worldObjects = worldObjects.Distinct().ToList();

            // Cache a map of containers to their resource count, in addition to the total count.
            // This is used to determine the size of the potential resource distribution.
            Dictionary<WorldObjectCore, uint> resourceCounts = new();
            uint totalAmount = 0;
            foreach (WorldObjectCore worldObject in worldObjects)
            {
                if (worldObject.resources.resources.ContainsKey(resource.Key))
                {
                    resourceCounts[worldObject] = worldObject.resources.resources[resource.Key];
                    totalAmount += worldObject.resources.resources[resource.Key];
                }
            }

            // Determine how much to distribute per container.
            uint amountPerContainer = (uint)(totalAmount / (float)worldObjects.Count);

            // Determine how much the current resource quantities
            // deviate from the average. This is used to determine
            // Whether to transfer resources or not.
            float averageDeviation =
                worldObjects
                    .Where(worldObject => resourceCounts.ContainsKey(worldObject))
                    .Select(worldObject =>
                        Math.Abs((int)resourceCounts[worldObject] - (int)amountPerContainer)
                    )
                    .ToList()
                    .Sum() / (float)worldObjects.Count;

            // If the average deviation is less than 1, then we are balanced enough.
            // This acts as a threshold to prevent infinite back and forth transfers.
            if (averageDeviation < 1)
            {
                return;
            }

            // If we have reached this point, then we have decided that we
            // need to balance the resources. So it's time to consume energy.
            try
            {
                this.battery.Energy -= 1;
            }
            catch (BatteryComponentCore.BatteryCapacityException)
            {
                return;
            }

            // Consume all the resources in every surrounding container.
            // While doing so, aggregate the total amount of resources consumed.
            foreach (WorldObjectCore worldObject in worldObjects)
            {
                if (worldObject.resources.resources.ContainsKey(resource.Key))
                {
                    worldObject.resources.resources[resource.Key] = 0;
                }
            }

            // Then distribute the resources evenly.
            foreach (WorldObjectCore worldObject in worldObjects)
            {
                worldObject.resources.resources[resource.Key] = amountPerContainer;
            }

            // If there is a remainder, distribute it evenly.
            uint remainder = totalAmount % (uint)worldObjects.Count;
            for (int i = 0; i < remainder; i++)
            {
                worldObjects[i].resources.resources[resource.Key] += 1;
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
                battery = battery,
                resources = resources,
                GridPosition = gridPosition,
                production = production,
            };
            TransferHubComponent transfer = new(gameController, core, battery);
            core.transferHub = transfer;
            core.guid = core.CreateGuid();
            gameController.worldObjects ??= new();
            if (!gameController.worldObjects.ContainsKey(core.GridPosition))
            {
                gameController.worldObjects[core.GridPosition] = new();
            }
            gameController.worldObjects[core.GridPosition][core.guid] = core;
            return core;
        }

        [Fact]
        public void TestOneOneTransferNOOP()
        {
            GameControllerCore gameController = new() { worldObjects = new() };
            WorldObjectCore core1 = this.WorldObject(gameController);
            WorldObjectCore core2 = this.WorldObject(gameController);

            core1.resources.CreateResources("wood", 100);
            core2.resources.CreateResources("wood", 100);

            core1.transferHub.Balance();

            Assert.Equal(100u, core1.resources.resources["wood"]);
            Assert.Equal(100u, core2.resources.resources["wood"]);
        }

        [Fact]
        public void TestOneZeroTransfer()
        {
            GameControllerCore gameController = new() { worldObjects = new() };
            WorldObjectCore core1 = this.WorldObject(gameController);
            WorldObjectCore core2 = this.WorldObject(gameController);

            core1.resources.CreateResources("wood", 100);
            core2.resources.CreateResources("wood", 0);

            core1.transferHub.Balance();

            Assert.Equal(50u, core1.resources.resources["wood"]);
            Assert.Equal(50u, core2.resources.resources["wood"]);
        }

        [Fact]
        public void TestTwoTwoTransfer()
        {
            GameControllerCore gameController = new() { worldObjects = new() };
            WorldObjectCore core1 = this.WorldObject(gameController);
            WorldObjectCore core2 = this.WorldObject(gameController);

            core1.resources.CreateResources("wood", 100);
            core1.resources.CreateResources("nails", 0);
            core2.resources.CreateResources("wood", 0);
            core2.resources.CreateResources("nails", 100);

            core1.transferHub.Balance();

            Assert.Equal(50u, core1.resources.resources["wood"]);
            Assert.Equal(50u, core1.resources.resources["nails"]);
            Assert.Equal(50u, core2.resources.resources["wood"]);
            Assert.Equal(50u, core2.resources.resources["nails"]);
        }

        [Fact]
        public void TestTruncation()
        {
            GameControllerCore gameController = new() { worldObjects = new() };
            WorldObjectCore core1 = this.WorldObject(gameController);
            WorldObjectCore core2 = this.WorldObject(gameController);
            WorldObjectCore core3 = this.WorldObject(gameController);

            core1.resources.CreateResources("wood", 100);
            core2.resources.CreateResources("wood", 0);
            core3.resources.CreateResources("wood", 0);

            core1.transferHub.Balance();

            Assert.Equal(
                new List<float>
                {
                    core1.resources.resources["wood"],
                    core2.resources.resources["wood"],
                    core3.resources.resources["wood"],
                }.Sum(),
                100u
            );
            Assert.Equal(34u, core1.resources.resources["wood"]);
            Assert.Equal(33u, core2.resources.resources["wood"]);
            Assert.Equal(33u, core3.resources.resources["wood"]);
        }

        [Fact]
        public void TestInfiniteTransferError()
        {
            GameControllerCore gameController = new() { worldObjects = new() };
            WorldObjectCore core1 = this.WorldObject(gameController);
            WorldObjectCore core2 = this.WorldObject(gameController);
            WorldObjectCore core3 = this.WorldObject(gameController);

            core1.resources.CreateResources("wood", 399);
            core2.resources.CreateResources("wood", 400);
            core3.resources.CreateResources("wood", 399);

            core1.transferHub.Balance();

            Assert.Equal(399u, core1.resources.resources["wood"]);
            Assert.Equal(400u, core2.resources.resources["wood"]);
            Assert.Equal(399u, core3.resources.resources["wood"]);

            core2.transferHub.Balance();

            Assert.Equal(399u, core1.resources.resources["wood"]);
            Assert.Equal(400u, core2.resources.resources["wood"]);
            Assert.Equal(399u, core3.resources.resources["wood"]);

            core3.transferHub.Balance();

            Assert.Equal(399u, core1.resources.resources["wood"]);
            Assert.Equal(400u, core2.resources.resources["wood"]);
            Assert.Equal(399u, core3.resources.resources["wood"]);
        }
    }
}
