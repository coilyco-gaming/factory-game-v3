// Inserter components query every nearby tile, once a tick,
// and insert matching items into the owner's resource inventory.


namespace Assets.Scripts.Components.Core
{
    using System.Collections.Generic;
    using System.Linq;
    using Assets.Scripts.Core;
    using Assets.Scripts.WorldObjects.Core;

    public class InserterComponentCore
    {
        public string resourceType = "";
        private ResourcesComponentCore resources;
        private BatteryComponentCore battery;
        private uint insertionRate = 0;

        public InserterComponentCore(
            BatteryComponentCore battery,
            ResourcesComponentCore resources,
            string resourceType,
            uint insertionRate
        )
        {
            this.battery =
                battery
                ?? throw new GameControllerCore.MisconfigurationException(
                    "Inserter component requires a battery component"
                );
            this.resources =
                resources
                ?? throw new GameControllerCore.MisconfigurationException(
                    "Inserter component requires a resources component"
                );
            this.resourceType = resourceType;
            this.insertionRate = insertionRate;
        }

        public void Insert(WorldObjectCore worldObject, GameControllerCore gameController)
        {
            List<WorldObjectCore> localWorldObjects = gameController.GetAdjacentWorldObjects(
                worldObject.GridPosition
            );
            List<ResourcesComponentCore> resources = localWorldObjects
                .Select(localWorldObject => localWorldObject.Resources)
                .Where(theseResources => theseResources != this.resources)
                .Where(theseResources => theseResources != null)
                .Where(theseResources =>
                    theseResources.Resources.GetValueOrDefault(this.resourceType, 0u) > 0
                )
                .Distinct()
                .ToList();

            foreach (ResourcesComponentCore resource in resources)
            {
                if (
                    resources != null
                    && this.resources != null
                    && this.battery != null
                    && this.battery.Energy > 0
                )
                {
                    try
                    {
                        this.resources.TakeResources(
                            resource,
                            this.resourceType,
                            this.insertionRate
                        );
                        this.battery.Energy -= 1;
                    }
                    catch (ResourcesComponentCore.ResourceException)
                    {
                        continue;
                    }
                }
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

    public class InserterComponentTest
    {
        private WorldObjectCore WorldObject(
            GameControllerCore gameController,
            System.Numerics.Vector2 gridPosition,
            List<InserterComponentCore> inserters = null,
            ResourcesComponentCore resources = null
        )
        {
            WorldObjectCore core = new(null)
            {
                Inserters = inserters,
                Resources = resources,
                GridPosition = gridPosition, // TODO: why can't these all be at the same grid position?
            };
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
        public void TestInsertCapacityOverflow()
        {
            GameControllerCore gameController = new();
            BatteryComponentCore battery = new(100, 100);

            // object 0
            ResourcesComponentCore resources0 = new(new TestResourcesGameContent(), 1, 1);
            resources0.CreateResources("wood", 1);
            WorldObjectCore worldObject0 = this.WorldObject(
                gameController,
                new System.Numerics.Vector2(0, 0),
                new List<InserterComponentCore>()
                {
                    new(
                        battery: battery,
                        resourceType: "wood",
                        insertionRate: 1,
                        resources: resources0
                    ),
                },
                resources0
            );

            // object 1
            ResourcesComponentCore resources1 = new(new TestResourcesGameContent(), 1, 1);
            WorldObjectCore worldObject1 = this.WorldObject(
                gameController,
                new System.Numerics.Vector2(1, 0),
                null,
                resources1
            );

            // logic under test
            worldObject0.Inserters[0].Insert(worldObject0, gameController);

            // assertions
            Assert.Equal(1u, worldObject0.Resources.Resources["wood"]);
            Assert.Equal(99u, battery.Energy);
            Assert.Equal(0u, worldObject1.Resources.Resources.GetValueOrDefault("wood", 0u));
        }

        [Fact]
        public void TestInsert()
        {
            GameControllerCore gameController = new();

            BatteryComponentCore battery = new(100, 100);

            // object 0
            ResourcesComponentCore resources0 = new(new TestResourcesGameContent(), 2, 2);
            resources0.CreateResources("wood", 1);
            WorldObjectCore worldObject0 = this.WorldObject(
                gameController,
                new System.Numerics.Vector2(0, 0),
                new List<InserterComponentCore>()
                {
                    new(
                        battery: battery,
                        resourceType: "wood",
                        insertionRate: 1,
                        resources: resources0
                    ),
                },
                resources0
            );

            // object 1
            ResourcesComponentCore resources1 = new(new TestResourcesGameContent(), 1, 1);
            resources1.CreateResources("wood", 1);
            WorldObjectCore worldObject1 = this.WorldObject(
                gameController,
                new System.Numerics.Vector2(1, 0),
                null,
                resources1
            );

            // logic under test
            worldObject0.Inserters[0].Insert(worldObject0, gameController);

            // assertions
            Assert.Equal(2u, worldObject0.Resources.Resources["wood"]);
            Assert.Equal(0u, worldObject1.Resources.Resources.GetValueOrDefault("wood", 0u));
        }

        [Fact]
        public void TestEmptyBatteryPreventsInsert()
        {
            GameControllerCore gameController = new();

            BatteryComponentCore battery = new(0, 100);

            // object 0
            ResourcesComponentCore resources0 = new(new TestResourcesGameContent(), 2, 2);
            resources0.CreateResources("wood", 1);
            WorldObjectCore worldObject0 = this.WorldObject(
                gameController,
                new System.Numerics.Vector2(0, 0),
                new List<InserterComponentCore>()
                {
                    new(
                        battery: battery,
                        resourceType: "wood",
                        insertionRate: 1,
                        resources: resources0
                    ),
                },
                resources0
            );

            // object 1
            ResourcesComponentCore resources1 = new(new TestResourcesGameContent(), 1, 1);
            resources1.CreateResources("wood", 1);
            WorldObjectCore worldObject1 = this.WorldObject(
                gameController,
                new System.Numerics.Vector2(1, 0),
                null,
                resources1
            );

            // logic under test
            worldObject0.Inserters[0].Insert(worldObject0, gameController);

            // assertions
            Assert.Equal(1u, worldObject0.Resources.Resources["wood"]);
            Assert.Equal(1u, worldObject1.Resources.Resources["wood"]);
        }

        [Fact]
        public void TestInsertMultiple()
        {
            GameControllerCore gameController = new();

            BatteryComponentCore battery = new(100, 100);

            // object 0
            ResourcesComponentCore resources0 = new(new TestResourcesGameContent(), 3, 3);
            resources0.CreateResources("wood", 1);
            WorldObjectCore worldObject0 = this.WorldObject(
                gameController,
                new System.Numerics.Vector2(0, 0),
                new List<InserterComponentCore>()
                {
                    new(
                        battery: battery,
                        resourceType: "wood",
                        insertionRate: 1,
                        resources: resources0
                    ),
                },
                resources0
            );

            // object 1
            ResourcesComponentCore resources1 = new(new TestResourcesGameContent(), 1, 1);
            resources1.CreateResources("wood", 1);
            WorldObjectCore worldObject1 = this.WorldObject(
                gameController,
                new System.Numerics.Vector2(1, 0),
                null,
                resources1
            );

            // object 2
            ResourcesComponentCore resources2 = new(new TestResourcesGameContent(), 1, 1);
            resources2.CreateResources("wood", 1);
            WorldObjectCore worldObject2 = this.WorldObject(
                gameController,
                new System.Numerics.Vector2(0, 1),
                null,
                resources2
            );

            // logic under test
            worldObject0.Inserters[0].Insert(worldObject0, gameController);

            // assertions
            Assert.Equal(3u, worldObject0.Resources.Resources["wood"]);
            Assert.Equal(0u, worldObject1.Resources.Resources.GetValueOrDefault("wood", 0u));
            Assert.Equal(0u, worldObject2.Resources.Resources.GetValueOrDefault("wood", 0u));
        }
    }
}
