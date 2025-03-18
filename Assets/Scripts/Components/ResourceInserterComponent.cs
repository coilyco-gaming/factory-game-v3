// Inserter components query every nearby tile, once a tick,
// and insert matching items into the owner's resource inventory.


namespace Assets.Scripts.Components.Core
{
    using System;
    using System.Collections.Generic;
    using System.Diagnostics;
    using System.Linq;
    using Assets.Scripts.Core;
    using UnityEngine;

    [Serializable]
    public class ResourceInserterComponentCore
    {
        private WorldObjectCore worldObject;
        public string resourceType = "";
        public uint insertionRate = 0;

        private ResourcesComponentCore resources;
        private BatteryComponentCore battery;

        public ResourceInserterComponentCore(
            WorldObjectCore worldObject,
            BatteryComponentCore battery,
            ResourcesComponentCore resources,
            string resourceType,
            uint insertionRate
        )
        {
            this.worldObject = worldObject;
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

        public List<Dictionary<uint, string>> Tick(GameControllerCore gameController)
        {
            using Activity activity = gameController.backref.ActivitySource.StartActivity(
                this.GetType().Name
            );
            activity.SetTag("WorldObjectType", this.worldObject.worldObjectType);
            activity.SetTag("tick", gameController.backref.TickCount);

            List<WorldObjectCore> localWorldObjects = gameController.GetAdjacentWorldObjects(
                this.worldObject.GridPosition
            );
            List<ResourcesComponentCore> resources = localWorldObjects
                .Select(localWorldObject => localWorldObject.resources)
                .Where(theseResources => theseResources != this.resources)
                .Where(theseResources => theseResources != null)
                .Where(theseResources =>
                    theseResources.resources.GetValueOrDefault(this.resourceType, 0u) > 0
                )
                .Distinct()
                .ToList();

            foreach (ResourcesComponentCore resource in resources)
            {
                if (
                    resources != null
                    && this.resources != null
                    && this.battery != null
                    && this.battery.Energy > 1
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
                    catch (ResourcesComponentCore.ResourceException) { }
                }
            }

            return new();
        }
    }
}

namespace Assets.Scripts.Components.Tests
{
    using System.Collections.Generic;
    using Assets.Scripts.Components.Core;
    using Assets.Scripts.Core;
    using Xunit;

    public class InserterComponentTest
    {
        private WorldObjectCore WorldObject(
            GameControllerCore gameController,
            System.Numerics.Vector2 gridPosition,
            List<ResourceInserterComponentCore> inserters = null,
            ResourcesComponentCore resources = null
        )
        {
            WorldObjectCore core = new(null)
            {
                resourceInserters = inserters,
                resources = resources,
                GridPosition = gridPosition, // TODO: why can't these all be at the same grid position?
            };
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
        public void TestInsertCapacityOverflow()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };

            BatteryComponentCore battery = new(new WorldObjectCore(null), 100, 100);

            // object 0
            ResourcesComponentCore resources0 = new(new TestResourcesGameContent(), 1, 1);
            resources0.CreateResources("wood", 1);
            WorldObjectCore worldObject0 = this.WorldObject(
                gameController,
                new System.Numerics.Vector2(0, 0),
                new List<ResourceInserterComponentCore>()
                {
                    new(
                        new WorldObjectCore(null),
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
            worldObject0.resourceInserters[0].Tick(gameController);

            // assertions
            Assert.Equal(1u, worldObject0.resources.resources["wood"]);
            Assert.Equal(100u, battery.Energy);
            Assert.Equal(0u, worldObject1.resources.resources.GetValueOrDefault("wood", 0u));
        }

        [Fact]
        public void TestInsert()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };

            BatteryComponentCore battery = new(new WorldObjectCore(null), 100, 100);

            // object 0
            ResourcesComponentCore resources0 = new(new TestResourcesGameContent(), 2, 2);
            resources0.CreateResources("wood", 1);
            WorldObjectCore worldObject0 = this.WorldObject(
                gameController,
                new System.Numerics.Vector2(0, 0),
                new List<ResourceInserterComponentCore>()
                {
                    new(
                        new WorldObjectCore(null),
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
            worldObject0.resourceInserters[0].Tick(gameController);

            // assertions
            Assert.Equal(2u, worldObject0.resources.resources["wood"]);
            Assert.Equal(0u, worldObject1.resources.resources.GetValueOrDefault("wood", 0u));
        }

        [Fact]
        public void TestBatteryMaxedAndStorageMaxed()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };

            BatteryComponentCore battery = new(new WorldObjectCore(null), 100, 100);

            // object 0
            ResourcesComponentCore resources0 = new(new TestResourcesGameContent(), 1, 1);
            resources0.CreateResources("wood", 1);
            WorldObjectCore worldObject0 = this.WorldObject(
                gameController,
                new System.Numerics.Vector2(0, 0),
                new List<ResourceInserterComponentCore>()
                {
                    new(
                        new WorldObjectCore(null),
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
            worldObject0.resourceInserters[0].Tick(gameController);

            // assertions
            Assert.Equal(1u, worldObject0.resources.resources["wood"]);
            Assert.Equal(1u, worldObject1.resources.resources["wood"]);
            Assert.Equal(100u, battery.Energy);
        }

        [Fact]
        public void TestEmptyBatteryPreventsInsert()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };

            BatteryComponentCore battery = new(new WorldObjectCore(null), 0, 100);

            // object 0
            ResourcesComponentCore resources0 = new(new TestResourcesGameContent(), 2, 2);
            resources0.CreateResources("wood", 1);
            WorldObjectCore worldObject0 = this.WorldObject(
                gameController,
                new System.Numerics.Vector2(0, 0),
                new List<ResourceInserterComponentCore>()
                {
                    new(
                        new WorldObjectCore(null),
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
            worldObject0.resourceInserters[0].Tick(gameController);

            // assertions
            Assert.Equal(1u, worldObject0.resources.resources["wood"]);
            Assert.Equal(1u, worldObject1.resources.resources["wood"]);
        }

        [Fact]
        public void TestInsertMultiple()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };

            BatteryComponentCore battery = new(new WorldObjectCore(null), 100, 100);

            // object 0
            ResourcesComponentCore resources0 = new(new TestResourcesGameContent(), 3, 3);
            resources0.CreateResources("wood", 1);
            WorldObjectCore worldObject0 = this.WorldObject(
                gameController,
                new System.Numerics.Vector2(0, 0),
                new List<ResourceInserterComponentCore>()
                {
                    new(
                        new WorldObjectCore(null),
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
            worldObject0.resourceInserters[0].Tick(gameController);

            // assertions
            Assert.Equal(3u, worldObject0.resources.resources["wood"]);
            Assert.Equal(0u, worldObject1.resources.resources.GetValueOrDefault("wood", 0u));
            Assert.Equal(0u, worldObject2.resources.resources.GetValueOrDefault("wood", 0u));
        }
    }
}
