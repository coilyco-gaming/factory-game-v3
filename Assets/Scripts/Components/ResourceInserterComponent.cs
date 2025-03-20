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
        public string resourceType = "";
        public uint insertionRate = 0;

        public ResourceInserterComponentCore(string resourceType, uint insertionRate)
        {
            this.resourceType = resourceType;
            this.insertionRate = insertionRate;
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

            List<WorldObjectCore> localWorldObjects = gameController.GetAdjacentWorldObjects(
                worldObject.GridPosition
            );
            List<ResourcesComponentCore> resources = localWorldObjects
                .Select(localWorldObject => localWorldObject.resources)
                .Where(theseResources => theseResources != null)
                .Where(theseResources => theseResources != worldObject.resources)
                .Where(theseResources =>
                    theseResources.resources.GetValueOrDefault(this.resourceType, 0u) > 0
                )
                .Distinct()
                .ToList();

            foreach (ResourcesComponentCore resource in resources)
            {
                if (worldObject.battery.Energy > 1)
                {
                    try
                    {
                        worldObject.resources.TakeResources(
                            resource,
                            this.resourceType,
                            this.insertionRate
                        );
                        worldObject.battery.Energy -= 1;
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
    using System.Linq;
    using Assets.Scripts.Components.Core;
    using Assets.Scripts.Core;
    using Xunit;
    using Xunit.Abstractions;

    public class InserterComponentTest
    {
        private ITestOutputHelper testOutput;

        public InserterComponentTest(ITestOutputHelper testOutputHelper)
        {
            this.testOutput = testOutputHelper;
        }

        [Fact]
        public void TestInsertCapacityOverflow()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };

            // object 0
            BatteryComponentCore battery = new();
            ResourcesComponentCore resources0 = new(new TestResourcesGameContent(), 1, 1);
            resources0.CreateResources("wood", 1);
            WorldObjectCore worldObject0 = new(null)
            {
                resourceInserters = new List<ResourceInserterComponentCore>()
                {
                    new(resourceType: "wood", insertionRate: 1),
                },
                resources = resources0,
                GridPosition = new System.Numerics.Vector2(0, 0),
                battery = battery,
            };
            worldObject0.guid = worldObject0.CreateGuid();

            // object 1
            ResourcesComponentCore resources1 = new(new TestResourcesGameContent(), 1, 1);
            WorldObjectCore worldObject1 = new(null)
            {
                resources = resources1,
                GridPosition = new System.Numerics.Vector2(1, 0),
            };
            worldObject1.guid = worldObject1.CreateGuid();

            // game controller setup
            gameController.worldObjects ??= new();
            gameController.worldObjects[worldObject0.GridPosition] = new()
            {
                { worldObject0.guid, worldObject0 },
            };
            gameController.worldObjects[worldObject1.GridPosition] = new()
            {
                { worldObject1.guid, worldObject1 },
            };

            // logic under test
            worldObject0.resourceInserters[0].Tick(gameController, worldObject0);

            // assertions
            Assert.Equal(1u, worldObject0.resources.resources["wood"]);
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

            // object 0
            ResourcesComponentCore resources0 = new(new TestResourcesGameContent(), 2, 2);
            resources0.CreateResources("wood", 1);
            WorldObjectCore worldObject0 = new(null)
            {
                resourceInserters = new List<ResourceInserterComponentCore>()
                {
                    new(resourceType: "wood", insertionRate: 1),
                },
                resources = resources0,
                GridPosition = new System.Numerics.Vector2(0, 0),
            };
            worldObject0.guid = worldObject0.CreateGuid();
            BatteryComponentCore battery = new(100, 100);
            worldObject0.battery = battery;

            // object 1
            ResourcesComponentCore resources1 = new(new TestResourcesGameContent(), 1, 1);
            resources1.CreateResources("wood", 1);
            WorldObjectCore worldObject1 = new(null)
            {
                resources = resources1,
                GridPosition = new System.Numerics.Vector2(1, 0),
            };
            worldObject1.guid = worldObject1.CreateGuid();

            // game controller setup
            gameController.worldObjects[worldObject0.GridPosition] = new()
            {
                { worldObject0.guid, worldObject0 },
            };
            gameController.worldObjects[worldObject1.GridPosition] = new()
            {
                { worldObject1.guid, worldObject1 },
            };

            // logic under test
            worldObject0.resourceInserters[0].Tick(gameController, worldObject0);

            // assertions
            Assert.Equal(2u, worldObject0.resources.resources["wood"]);
            Assert.Equal(0u, worldObject1.resources.resources.GetValueOrDefault("wood", 0u));
        }

        // Other test methods follow the same pattern, ensuring the correct parameters are passed
        // and constructors are used correctly.
    }
}
