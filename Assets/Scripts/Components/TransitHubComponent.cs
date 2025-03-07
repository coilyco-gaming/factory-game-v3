// A type of component that tries to balance all available
// resources between all connected entities.

namespace Assets.Scripts.Components.Core
{
    using System.Collections.Generic;
    using System.Linq;
    using Assets.Scripts.Core;
    using Assets.Scripts.WorldObjects.Core;

    public class TransitHubComponent
    {
        private WorldObjectCore core;
        private BatteryComponentCore battery;
        private GameContent gameContent;

        public TransitHubComponent(
            GameContent gameContent,
            WorldObjectCore core,
            BatteryComponentCore battery
        )
        {
            this.gameContent = gameContent;
            this.core = core;
            this.battery =
                battery
                ?? throw new GameControllerCore.MisconfigurationException(
                    "Transfer Hub component requires a battery component"
                );
        }

        public void Balance(GameControllerCore gameController)
        {
            List<WorldObjectCore> worldObjects = gameController
                .GetAdjacentWorldObjects(this.core.GridPosition)
                .Where(worldObject => worldObject.resources != null)
                .Distinct()
                .ToList();

            List<ProductionComponentCore> localProductionComponents = worldObjects
                .Select(worldObject => worldObject.production)
                .Where(production => production != null)
                .Distinct()
                .ToList();

            List<string> localIngredients = localProductionComponents
                .Where(production => production?.ProductItem?.Ingredients != null)
                .SelectMany(production => production.ProductItem.Ingredients.Keys)
                .Where(ingredient => ingredient != null)
                .Distinct()
                .ToList();

            Dictionary<string, uint> localIngredientsCounts = localIngredients.ToDictionary(
                ingredient => ingredient,
                ingredient =>
                    (uint)
                        localProductionComponents
                            .Where(production => production?.ProductItem?.Ingredients != null)
                            .SelectMany(production => production?.ProductItem?.Ingredients)
                            .Sum(ingredient => ingredient.Value)
            );

            // TODO: simply iterate every resource in the game
            foreach (KeyValuePair<string, uint> resource in localIngredientsCounts)
            {
                this.BalanceResource(gameController, resource);
            }
        }

        private void BalanceResource(
            GameControllerCore gameController,
            KeyValuePair<string, uint> resource
        )
        {
            // Local world objects whose factories produce this resource.
            List<WorldObjectCore> producers = gameController
                .GetAdjacentWorldObjects(this.core.GridPosition) // TODO: iterate 2 blocks out, instead of 1
                .Where(worldObject => worldObject?.production?.Product == resource.Key)
                .Distinct()
                .ToList();

            // Local world objects whose factories consume this resource.
            List<WorldObjectCore> consumers = gameController
                .GetAdjacentWorldObjects(this.core.GridPosition) // TODO: iterate 2 blocks out, instead of 1
                .Where(worldObject =>
                    worldObject?.production?.ProductItem?.Ingredients != null
                    && new List<string>(
                        worldObject?.production?.ProductItem?.Ingredients.Keys
                    ).Contains(resource.Key)
                )
                .Distinct()
                .ToList();

            // Check some exit conditions.
            if (producers.Count == 0 || consumers.Count == 0 || this.battery.Energy < 1)
            {
                return;
            }

            // Use power
            try
            {
                this.battery.Energy -= 1;
            }
            catch (BatteryComponentCore.BatteryCapacityException)
            {
                return;
            }

            // For every producer, try and consume a stack of the resource.
            uint resourcesToDistribute = 0;
            foreach (WorldObjectCore producer in producers)
            {
                try
                {
                    producer.resources.ConsumeResources(
                        resource.Key,
                        this.gameContent.Items[resource.Key].StackSize
                    );
                    resourcesToDistribute += this.gameContent.Items[resource.Key].StackSize;
                }
                catch (ResourcesComponentCore.ResourceException) { }
            }

            // Determine how many resources each consumer should get.
            uint resourcesPerConsumer = 0;
            if (consumers.Count > 0)
            {
                resourcesPerConsumer = (uint)(resourcesToDistribute / (float)consumers.Count);
            }

            // For every consumer, force create a batch of the resource.
            foreach (WorldObjectCore consumer in consumers)
            {
                try
                {
                    consumer.resources.ForceCreateResources(resource.Key, resourcesPerConsumer);
                }
                catch (ResourcesComponentCore.ResourceException) { }
            }
        }
    }
}

namespace Assets.Scripts.Components.Tests
{
    using System.Collections.Generic;
    using System.Numerics;
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

    public class TransitHubComponentTests
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
            TransitHubComponent hub = new(new TestGameContent(), core, battery);
            core.hub = hub;
            core.guid = core.CreateGuid();
            gameController.worldObjects ??= new();
            if (!gameController.worldObjects.ContainsKey(core.GridPosition))
            {
                gameController.worldObjects[core.GridPosition] = new();
            }
            gameController.worldObjects[core.GridPosition][core.guid] = core;
            return core;
        }
    }
}
