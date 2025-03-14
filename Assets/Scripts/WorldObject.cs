namespace Assets.Scripts.Core
{
    using System;
    using System.Collections.Generic;
    using System.Linq;
    using Assets.Scripts.Components.Core;
    using Assets.Scripts.Unity;

    [Serializable]
    public class WorldObjectCore
    {
        // PROPERTIES //

        public float ZIndex => 1;
        public static uint MaxAlerts = 10;

        // TODO: turn all of these into fields

        // TODO: make each component manage its state via a "data" field on the world object

        // TODO: add odin inspector to all of the serializable classes
        // https://odininspector.com/tutorials

        public MovementComponentCore movement;
        public BatteryComponentCore battery;
        public ResourcesComponentCore resources;
        public List<ResourceInserterComponentCore> resourceInserters;
        public List<DispatchComponentCore> dispatchers;
        public List<DispatchReceiverComponentCore> dispatchReceivers;
        public List<DeploymentComponentCore> deployments;
        public ResourceRetrieverCore resourceRetriever;
        public ProductionComponentCore production;
        public PowerComponentCore power;
        public MiningComponentCore mining;
        public string guid;
        public string worldObjectType;
        public string targetType;
        public string targetSubType;
        public WorldObject backref;
        public bool mobile = false;
        public bool passThrough = false;
        private List<Dictionary<uint, string>> alerts = new();
        public System.Numerics.Vector2 gridPosition;

        public System.Numerics.Vector2 GridPosition
        {
            get => this.gridPosition;
            set => this.gridPosition = value;
        }

        public List<Dictionary<uint, string>> Alerts
        {
            get => this.alerts;
            set
            {
                // Match on the string value of the alerts dictionary
                // Then replace the int value with the current tick.
                // This produces the following effect:
                //
                //   { 1: "I'm broken and need repairs!" } =>
                //   { 2: "I'm broken and need repairs!" }
                //
                // This happens without creating a new line in the alerts list.

                List<Dictionary<uint, string>> newAlerts = this.alerts ??= new();

                // For every input alert
                foreach (Dictionary<uint, string> inputAlert in value)
                {
                    // If the alert is already in the list
                    bool skip = false;

                    foreach (Dictionary<uint, string> existingAlert in newAlerts)
                    {
                        // If the alert message is the same
                        if (existingAlert.Values.First() == inputAlert.Values.First())
                        {
                            // Replace the alert with the new tick
                            newAlerts.Remove(existingAlert);
                            newAlerts.Add(
                                new() { { inputAlert.Keys.First(), inputAlert.Values.First() } }
                            );
                            skip = true;
                            break;
                        }
                    }
                    if (!skip)
                    {
                        newAlerts.Add(inputAlert);
                    }
                }

                // Clip the alert list
                this.alerts = newAlerts.TakeLast((int)MaxAlerts).ToList();
            }
        }

        // FUNCTIONS //

        public WorldObjectCore(WorldObject backref)
        {
            this.backref = backref;
        }

        public void Instantiate(SpawnQueueItem spawnQueueItem, GameContent gameContent)
        {
            this.GridPosition = spawnQueueItem.gridPosition;
            this.targetType = spawnQueueItem.targetType;
            this.targetSubType = spawnQueueItem.targetSubType;
            this.guid = this.CreateGuid();
        }

        public void PostInstantiate(SpawnQueueItem spawnQueueItem, GameContent gameContent)
        {
            if (this.resources == null && spawnQueueItem.resources != null)
            {
                throw new GameControllerCore.MisconfigurationException(
                    $"WorldObject {this.worldObjectType} has no resources component but has resources."
                );
            }
            if (spawnQueueItem.resources != null)
            {
                this.resources.resources = spawnQueueItem.resources;
            }
        }

        public string CreateGuid()
        {
            long time = DateTime.UtcNow.Ticks;
            byte[] guidBytes = System.Guid.NewGuid().ToByteArray();
            byte[] counterBytes = BitConverter.GetBytes(time);
            Array.Copy(counterBytes, 0, guidBytes, guidBytes.Length - 8, 8);
            Guid timeOffsetGuid = new(guidBytes);
            string guidString = timeOffsetGuid.ToString();
            return guidString;
        }
    }
}

namespace Assets.Scripts.Tests
{
    using System.Collections.Generic;
    using Assets.Scripts.Core;
    using Xunit;
    using Xunit.Abstractions;

    public class WorldObjectCoreTest
    {
        private ITestOutputHelper testOutput;

        public WorldObjectCoreTest(ITestOutputHelper testOutputHelper)
        {
            this.testOutput = testOutputHelper;
        }

        [Fact]
        public void TestOneAlert()
        {
            WorldObjectCore core = new(null)
            {
                Alerts = new List<Dictionary<uint, string>>()
                {
                    new() { { 10, "I'm broken and need repairs!" } },
                },
            };
            Assert.Equal(1, core.Alerts.Count);
        }

        [Fact]
        public void TestTwoOfSameAlert()
        {
            WorldObjectCore core = new(null)
            {
                Alerts = new List<Dictionary<uint, string>>()
                {
                    new() { { 10, "I'm broken and need repairs!" } },
                },
            };
            core.Alerts = new List<Dictionary<uint, string>>()
            {
                new() { { 20, "I'm broken and need repairs!" } },
            };
            Assert.Equal(1, core.Alerts.Count);
        }

        [Fact]
        public void TestTwoDifferentAlert()
        {
            WorldObjectCore core = new(null)
            {
                Alerts = new List<Dictionary<uint, string>>()
                {
                    new() { { 10, "I'm broken and need repairs!" } },
                },
            };
            core.Alerts = new List<Dictionary<uint, string>>()
            {
                new() { { 20, "I'm out of power!" } },
            };
            Assert.Equal(2, core.Alerts.Count);
        }

        [Fact]
        public void TestTwoAtOnce()
        {
            WorldObjectCore core = new(null)
            {
                Alerts = new List<Dictionary<uint, string>>()
                {
                    new() { { 10, "I'm broken and need repairs!" } },
                    new() { { 20, "I'm out of power!" } },
                },
            };
            Assert.Equal(2, core.Alerts.Count);
        }

        [Fact]
        public void TestTwoThenOneMore()
        {
            WorldObjectCore core = new(null)
            {
                Alerts = new List<Dictionary<uint, string>>()
                {
                    new() { { 10, "I'm broken and need repairs!" } },
                    new() { { 20, "I'm out of power!" } },
                },
            };
            core.Alerts = new List<Dictionary<uint, string>>()
            {
                new() { { 30, "I can't move!" } },
            };
            Assert.Equal(3, core.Alerts.Count);
        }
    }
}

namespace Assets.Scripts.Unity
{
    using System;
    using Assets.Scripts.Components.Core;
    using Assets.Scripts.Core;
    using UnityEngine;

    [Serializable]
    public class WorldObject : MonoBehaviour
    {
        public WorldObjectCore core;

        // PROPERTIES //

        public virtual float ZIndex => 1;

        public string Guid
        {
            get => this.core.guid;
            set => this.core.guid = value;
        }

        public string WorldObjectType
        {
            get => this.core.worldObjectType;
            set => this.core.worldObjectType = value;
        }

        public System.Numerics.Vector2 GridPosition
        {
            get => this.core.GridPosition;
            set
            {
                this.core.GridPosition = value;
                this.transform.localPosition = new Vector3(value.X, value.Y, -this.ZIndex);
            }
        }

        public virtual StatusDataComponentCore StatusData =>
            new()
            { //
                Name = Util.HumanizedString(this.WorldObjectType),
            };

        // FUNCTIONS //

        public virtual void Tick(GameController gameController) { }

        public virtual void Instantiate(SpawnQueueItem spawnQueueItem, GameContent gameContent)
        {
            this.core = new WorldObjectCore(this);
            this.core.Instantiate(spawnQueueItem, gameContent);
            this.GridPosition = spawnQueueItem.gridPosition; // This is a special case because it sets the transform position
            this.WorldObjectType = this.transform.name.Replace("(Clone)", "");
            this.SetName();
        }

        public virtual void PostInstantiate(SpawnQueueItem spawnQueueItem, GameContent gameContent)
        {
            this.core.PostInstantiate(spawnQueueItem, gameContent);
        }

        public void SetName()
        {
            this.transform.name =
                $"{this.WorldObjectType} ({this.GridPosition.X}, {this.GridPosition.Y})";
        }
    }
}
