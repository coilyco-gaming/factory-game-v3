namespace Assets.Scripts.Components.Core
{
    using System;
    using System.Collections.Generic;
    using Assets.Scripts.Core;

    public class ResourcesComponentCore
    {
        // FIELDS //
        public uint weightCapacity = 0;
        public uint volumeCapacity = 0;
        private GameContent GameContent;

        // PROPERTIES //

        // TODO: reserve storage for certain kinds of resources

        public Dictionary<string, uint> Resources { get; private set; } = new();

        public virtual Dictionary<string, string> ResourceInfo
        {
            get
            {
                Dictionary<string, string> info = new();
                foreach (KeyValuePair<string, uint> resource in this.Resources)
                {
                    info.Add(resource.Key, resource.Value.ToString());
                }
                return info;
            }
        }

        public uint TotalResources
        {
            get
            {
                uint total = 0;
                foreach (uint resource in this.Resources.Values)
                {
                    total += resource;
                }
                return total;
            }
        }

        public bool HasResources => this.TotalResources > 0;

        public uint UsedWeightCapacity
        {
            get
            {
                uint totalWeight = 0;
                foreach (KeyValuePair<string, uint> resourcePair in this.Resources)
                {
                    GameContent.Item item = this.GameContent.Items.GetValueOrDefault(
                        resourcePair.Key
                    );
                    totalWeight += item.Weight * resourcePair.Value;
                }
                return totalWeight;
            }
        }

        public uint RemainingWeightCapacity => this.weightCapacity - this.UsedWeightCapacity;

        private double UsedWeightPercent =>
            this.weightCapacity != 0
                ? Math.Round(this.UsedWeightCapacity / (double)this.weightCapacity, 2) * 100
                : 0;

        public string UsedWeightString => $"{this.UsedWeightPercent}%";

        public uint UsedVolumeCapacity
        {
            get
            {
                uint totalVolume = 0;
                foreach (KeyValuePair<string, uint> resourcePair in this.Resources)
                {
                    GameContent.Item item = this.GameContent.Items.GetValueOrDefault(
                        resourcePair.Key
                    );
                    totalVolume += item.Volume * resourcePair.Value;
                }
                return totalVolume;
            }
        }

        public uint RemainingVolumeCapacity => this.volumeCapacity - this.UsedVolumeCapacity;

        private double UsedVolumePercent =>
            this.volumeCapacity != 0
                ? Math.Round(this.UsedVolumeCapacity / (double)this.volumeCapacity, 2) * 100
                : 0;

        public string UsedVolumeString => $"{this.UsedVolumePercent}%";

        // CLASSES //

        public class ResourceException : Exception
        {
            public ResourceException(string message)
                : base(message) { }
        }

        public class ResourceContainerException : ResourceException
        {
            public ResourceContainerException(string message)
                : base(message) { }
        }

        public class ResourceWeightCapacityException : ResourceException
        {
            public ResourceWeightCapacityException(string message)
                : base(message) { }
        }

        public class ResourceVolumeCapacityException : ResourceException
        {
            public ResourceVolumeCapacityException(string message)
                : base(message) { }
        }

        public class ResourceQuantityException : ResourceException
        {
            public ResourceQuantityException(string message)
                : base(message) { }
        }

        // CONSTRUCTORS //

        private ResourcesComponentCore() { }

        public ResourcesComponentCore(
            GameContent gameContent,
            uint weightCapacity = 100,
            uint volumeCapacity = 100
        )
        {
            this.GameContent = gameContent;
            this.weightCapacity = weightCapacity;
            this.volumeCapacity = volumeCapacity;
        }

        // FUNCTIONS //

        public bool CreateResources(string resourceName, uint amountToCreate)
        {
            // So many null checks... @_@
            GameContent.Item item =
                this.GameContent?.Items?.GetValueOrDefault(
                    resourceName ?? "",
                    new GameContent.Item("")
                ) ?? new GameContent.Item("");

            uint weightToCreate = amountToCreate * item.Weight;
            uint volumeToCreate = amountToCreate * item.Volume;
            uint currentResources = this.Resources.GetValueOrDefault(resourceName, 0u);

            if (this.RemainingWeightCapacity < weightToCreate)
            {
                amountToCreate = (uint)(this.RemainingWeightCapacity / (float)item.Weight);
                this.Resources[resourceName] = currentResources + amountToCreate;
                throw new ResourceWeightCapacityException(
                    $"Not enough weight capacity to create {amountToCreate} {resourceName}"
                );
            }

            if (this.RemainingVolumeCapacity < volumeToCreate)
            {
                amountToCreate = (uint)(this.RemainingVolumeCapacity / (float)item.Volume);
                this.Resources[resourceName] = currentResources + amountToCreate;
                throw new ResourceVolumeCapacityException(
                    $"Not enough volume capacity to create {amountToCreate} {resourceName}"
                );
            }

            this.Resources[resourceName] = currentResources + amountToCreate;
            return true;
        }

        public bool ConsumeResources(string resourceName, uint amountToConsume)
        {
            uint availableResources = this.Resources.GetValueOrDefault(resourceName, 0u);
            if (availableResources < amountToConsume)
            {
                throw new ResourceException(
                    $"Does not have {amountToConsume} {resourceName} to consume"
                );
            }

            if (amountToConsume != 0)
            {
                this.Resources[resourceName] -= amountToConsume;
            }
            return true;
        }

        public void GiveResources(
            ResourcesComponentCore target,
            string resourceName,
            uint amountToGive
        )
        {
            if (target == null)
            {
                throw new ResourceContainerException("Nowhere to give resources");
            }

            uint availableResources = this.Resources.GetValueOrDefault(resourceName ?? "", (uint)0);

            // So many null checks... @_@
            GameContent.Item item =
                this.GameContent?.Items?.GetValueOrDefault(
                    resourceName ?? "",
                    new GameContent.Item("")
                ) ?? new GameContent.Item("");

            uint weightToGive = amountToGive * item.Weight;
            uint volumeToGive = amountToGive * item.Volume;
            uint currentResources = target.Resources.GetValueOrDefault(resourceName, (uint)0);

            if (availableResources == 0)
            {
                throw new ResourceQuantityException($"Does not have {resourceName} to give");
            }

            if (availableResources < amountToGive)
            {
                this.GiveResources(target, resourceName, availableResources);
                throw new ResourceQuantityException(
                    $"Does not have {amountToGive} {resourceName} to give"
                );
            }

            if (target.RemainingWeightCapacity < weightToGive)
            {
                amountToGive = (uint)(target.RemainingWeightCapacity / (float)item.Weight);
                this.Resources[resourceName] -= amountToGive;
                target.Resources[resourceName] = currentResources + amountToGive;
                throw new ResourceWeightCapacityException(
                    $"Not enough weight capacity to give {amountToGive} {resourceName}"
                );
            }

            if (target.RemainingVolumeCapacity < volumeToGive)
            {
                amountToGive = (uint)(target.RemainingVolumeCapacity / (float)item.Volume);
                this.Resources[resourceName] -= amountToGive;
                target.Resources[resourceName] = currentResources + amountToGive;
                throw new ResourceVolumeCapacityException(
                    $"Not enough volume capacity to give {amountToGive} {resourceName}"
                );
            }

            this.Resources[resourceName] -= amountToGive;
            target.Resources[resourceName] = currentResources + amountToGive;
        }

        public void TakeResouces(
            ResourcesComponentCore target,
            string resourceName,
            uint amountToTake
        )
        {
            // TODO: catch the exceptions, emit them as alerts on both world objects, then rethrow exception
            if (target == null)
            {
                throw new ResourceContainerException("No resources to take");
            }
            target.GiveResources(this, resourceName, amountToTake);
        }
    }
}

#if UNITY_6000
namespace Assets.Scripts.Components.Unity
{
    using System.Collections.Generic;
    using Assets.Scripts.Components.Core;
    using Assets.Scripts.Unity;
    using UnityEngine;

    public class ResourcesComponent : MonoBehaviour
    {
        // FIELDS //

        public ResourcesComponentCore core;

        // PROPERTIES //

        // TODO: allow viewing these values in the unity inspector somehow
        public Dictionary<string, uint> Resources => this.core.Resources;
        public string UsedVolumeString => this.core.UsedVolumeString;
        public string UsedWeightString => this.core.UsedWeightString;

        public virtual Dictionary<string, string> ResourceInfo => this.core.ResourceInfo;

        public uint TotalResources => this.core.TotalResources;

        public bool HasResources => this.core.HasResources;

        // FUNCTIONS //

        public void Instantiate(uint weightCapacity = 100, uint volumeCapacity = 100) =>
            this.core = new(
                gameContent: new FactoryGameContent(),
                weightCapacity: weightCapacity,
                volumeCapacity: volumeCapacity
            );

        public bool CreateResources(string resourceName, uint amountToCreate) =>
            this.core.CreateResources(resourceName, amountToCreate);

        public bool ConsumeResources(string resourceName, uint amountToConsume) =>
            this.core.ConsumeResources(resourceName, amountToConsume);

        public void GiveResources(
            ResourcesComponent target,
            string resourceName,
            uint amountToGive
        ) => this.core.GiveResources(target.core, resourceName, amountToGive);

        public void TakeResouces(
            ResourcesComponent target,
            string resourceName,
            uint amountToTake
        ) => this.core.TakeResouces(target.core, resourceName, amountToTake);
    }
}
#endif

namespace Assets.Scripts.Components.Tests
{
    using System.Collections.Generic;
    using Assets.Scripts.Components.Core;
    using Assets.Scripts.Core;
    using Xunit;

    public class TestResourcesGameContent : GameContent
    {
        public override Dictionary<string, Item> Items { get; } =
            new()
            {
                { "wood", new Item("wood") },
                { "stone", new Item("stone") },
                { "iron", new Item("iron") },
                { "coal", new Item("coal") },
                { "sunlight", new Item("sunlight", 0, 0) },
                { "large", new Item("large", volume: 1000) },
                { "heavy", new Item("heavy", weight: 1000) },
            };
    }

    public class ResourcesComponentTest
    {
        [Fact]
        public void TestFieldZeroStates()
        {
            ResourcesComponentCore resourcesComponent = new(new TestResourcesGameContent());
            Assert.Equal((uint)0, resourcesComponent.TotalResources);
            Assert.Equal(resourcesComponent.Resources.Count, 0);
            Assert.Equal(resourcesComponent.ResourceInfo.Count, 0);
            Assert.False(resourcesComponent.HasResources);
        }

        [Fact]
        public void TestResourcesOnInit()
        {
            ResourcesComponentCore resourcesComponent = new(new TestResourcesGameContent());
            resourcesComponent.CreateResources("wood", 10);
            resourcesComponent.CreateResources("stone", 20);
            resourcesComponent.CreateResources("iron", 30);
            Assert.Equal((uint)60, resourcesComponent.TotalResources);
            Assert.Equal(resourcesComponent.Resources.Count, 3);
            Assert.Equal(resourcesComponent.ResourceInfo.Count, 3);
            Assert.True(resourcesComponent.HasResources);
        }

        [Fact]
        public void TestConsumeResources()
        {
            ResourcesComponentCore resourcesComponent = new(new TestResourcesGameContent());
            resourcesComponent.CreateResources("wood", 10);
            resourcesComponent.CreateResources("stone", 20);
            resourcesComponent.CreateResources("iron", 30);
            resourcesComponent.ConsumeResources("wood", 5);
            Assert.Equal((uint)55, resourcesComponent.TotalResources);
            Assert.Equal((uint)5, resourcesComponent.Resources["wood"]);
        }

        [Fact]
        public void TestGiveAndTake()
        {
            ResourcesComponentCore resourcesComponent = new(new TestResourcesGameContent());
            resourcesComponent.CreateResources("wood", 10);
            resourcesComponent.CreateResources("stone", 20);
            resourcesComponent.CreateResources("iron", 30);

            ResourcesComponentCore targetResourcesComponent = new(new TestResourcesGameContent());
            targetResourcesComponent.CreateResources("wood", 5);
            targetResourcesComponent.CreateResources("stone", 10);
            targetResourcesComponent.CreateResources("iron", 15);

            resourcesComponent.GiveResources(targetResourcesComponent, "wood", 5);
            Assert.Equal((uint)55, resourcesComponent.TotalResources);
            Assert.Equal((uint)5, resourcesComponent.Resources["wood"]);
            Assert.Equal((uint)35, targetResourcesComponent.TotalResources);
            Assert.Equal((uint)10, targetResourcesComponent.Resources["wood"]);

            resourcesComponent.TakeResouces(targetResourcesComponent, "wood", 5);
            Assert.Equal((uint)60, resourcesComponent.TotalResources);
            Assert.Equal((uint)10, resourcesComponent.Resources["wood"]);
            Assert.Equal((uint)30, targetResourcesComponent.TotalResources);
            Assert.Equal((uint)5, targetResourcesComponent.Resources["wood"]);
        }

        [Fact]
        public void TestTakeFromNull()
        {
            ResourcesComponentCore resourcesComponent = new(new TestResourcesGameContent());
            resourcesComponent.CreateResources("wood", 10);
            resourcesComponent.CreateResources("stone", 20);
            resourcesComponent.CreateResources("iron", 30);

            Assert.Throws<ResourcesComponentCore.ResourceContainerException>(
                () => resourcesComponent.TakeResouces(null, "wood", 5)
            );
        }

        [Fact]
        public void TestGiveToNull()
        {
            ResourcesComponentCore resourcesComponent = new(new TestResourcesGameContent());
            resourcesComponent.CreateResources("wood", 10);
            resourcesComponent.CreateResources("stone", 20);
            resourcesComponent.CreateResources("iron", 30);

            Assert.Throws<ResourcesComponentCore.ResourceContainerException>(
                () => resourcesComponent.GiveResources(null, "wood", 5)
            );
        }

        [Fact]
        public void TestNotEnoughResourcesToGive()
        {
            ResourcesComponentCore resourcesComponent = new(new TestResourcesGameContent());

            ResourcesComponentCore targetResourcesComponent = new(new TestResourcesGameContent());
            targetResourcesComponent.CreateResources("wood", 5);
            targetResourcesComponent.CreateResources("stone", 10);
            targetResourcesComponent.CreateResources("iron", 15);

            Assert.Throws<ResourcesComponentCore.ResourceQuantityException>(
                () => resourcesComponent.GiveResources(targetResourcesComponent, "wood", 15)
            );
        }

        [Fact]
        public void TestNotEnoughCapacityToRecieveWeight()
        {
            ResourcesComponentCore resourcesComponent = new(new TestResourcesGameContent());
            resourcesComponent.CreateResources("wood", 20);

            ResourcesComponentCore targetResourcesComponent = new(
                new TestResourcesGameContent(),
                weightCapacity: 100,
                volumeCapacity: 200
            );
            targetResourcesComponent.CreateResources("wood", 90);

            Assert.Equal(90u, targetResourcesComponent.Resources["wood"]);
            Assert.Equal(100u, targetResourcesComponent.weightCapacity);
            Assert.Equal(10u, targetResourcesComponent.RemainingWeightCapacity);
            Assert.Equal("90%", targetResourcesComponent.UsedWeightString);
            Assert.Throws<ResourcesComponentCore.ResourceWeightCapacityException>(
                () => resourcesComponent.GiveResources(targetResourcesComponent, "wood", 20)
            );
        }

        [Fact]
        public void TestNotEnoughCapacityToRecieveVolume()
        {
            ResourcesComponentCore resourcesComponent = new(new TestResourcesGameContent());
            resourcesComponent.CreateResources("wood", 20);

            ResourcesComponentCore targetResourcesComponent = new(
                new TestResourcesGameContent(),
                weightCapacity: 200,
                volumeCapacity: 100
            );
            targetResourcesComponent.CreateResources("wood", 90);

            Assert.Equal(90u, targetResourcesComponent.Resources["wood"]);
            Assert.Equal(100u, targetResourcesComponent.volumeCapacity);
            Assert.Equal(10u, targetResourcesComponent.RemainingVolumeCapacity);
            Assert.Equal("90%", targetResourcesComponent.UsedVolumeString);
            Assert.Throws<ResourcesComponentCore.ResourceVolumeCapacityException>(
                () => resourcesComponent.GiveResources(targetResourcesComponent, "wood", 20)
            );
        }

        [Fact]
        public void TestInfiniteSunlight()
        {
            ResourcesComponentCore resourcesComponent = new(new TestResourcesGameContent());
            resourcesComponent.CreateResources("sunlight", uint.MaxValue);
            Assert.Equal(uint.MaxValue, resourcesComponent.TotalResources);
        }

        [Fact]
        public void TestLarge()
        {
            ResourcesComponentCore resourcesComponent = new(
                new TestResourcesGameContent(),
                weightCapacity: 100,
                volumeCapacity: 100
            );
            Assert.Throws<ResourcesComponentCore.ResourceVolumeCapacityException>(
                () => resourcesComponent.CreateResources("large", 1)
            );
            Assert.Equal("0%", resourcesComponent.UsedVolumeString);
            Assert.Equal("0%", resourcesComponent.UsedWeightString);
        }

        [Fact]
        public void TestGiveOneLarge()
        {
            ResourcesComponentCore resourcesComponent = new(
                new TestResourcesGameContent(),
                weightCapacity: 1000,
                volumeCapacity: 1000
            );
            resourcesComponent.CreateResources("large", 1);
            ResourcesComponentCore targetResourcesComponent = new(
                new TestResourcesGameContent(),
                weightCapacity: 100,
                volumeCapacity: 100
            );
            Assert.Equal("100%", resourcesComponent.UsedVolumeString);
            Assert.Equal("0%", targetResourcesComponent.UsedVolumeString);
            Assert.Throws<ResourcesComponentCore.ResourceVolumeCapacityException>(
                () => resourcesComponent.GiveResources(targetResourcesComponent, "large", 1)
            );
            Assert.Equal("100%", resourcesComponent.UsedVolumeString);
            Assert.Equal("0%", targetResourcesComponent.UsedVolumeString);
        }

        [Fact]
        public void TestHeavy()
        {
            ResourcesComponentCore resourcesComponent = new(
                new TestResourcesGameContent(),
                weightCapacity: 100,
                volumeCapacity: 100
            );
            Assert.Throws<ResourcesComponentCore.ResourceWeightCapacityException>(
                () => resourcesComponent.CreateResources("heavy", 1)
            );
            Assert.Equal("0%", resourcesComponent.UsedVolumeString);
            Assert.Equal("0%", resourcesComponent.UsedWeightString);
        }

        [Fact]
        public void TestGiveOneHeavy()
        {
            ResourcesComponentCore resourcesComponent = new(
                new TestResourcesGameContent(),
                weightCapacity: 1000,
                volumeCapacity: 1000
            );
            resourcesComponent.CreateResources("heavy", 1);
            ResourcesComponentCore targetResourcesComponent = new(
                new TestResourcesGameContent(),
                weightCapacity: 100,
                volumeCapacity: 100
            );
            Assert.Equal("100%", resourcesComponent.UsedWeightString);
            Assert.Equal("0%", targetResourcesComponent.UsedWeightString);
            Assert.Throws<ResourcesComponentCore.ResourceWeightCapacityException>(
                () => resourcesComponent.GiveResources(targetResourcesComponent, "heavy", 1)
            );
            Assert.Equal("100%", resourcesComponent.UsedWeightString);
            Assert.Equal("0%", targetResourcesComponent.UsedWeightString);
        }

        [Fact]
        public void TestFractionalPercents()
        {
            ResourcesComponentCore resourcesComponent = new(
                new TestResourcesGameContent(),
                volumeCapacity: 1000,
                weightCapacity: 1000
            );
            resourcesComponent.CreateResources("wood", 333);
            Assert.Equal("33%", resourcesComponent.UsedVolumeString);
            Assert.Equal("33%", resourcesComponent.UsedWeightString);
        }

        [Fact]
        public void TestOversupplyCreatesPartial()
        {
            ResourcesComponentCore resourcesComponent = new(
                new TestResourcesGameContent(),
                volumeCapacity: 100,
                weightCapacity: 100
            );
            Assert.Throws<ResourcesComponentCore.ResourceWeightCapacityException>(
                () => resourcesComponent.CreateResources("wood", 1000)
            );
            Assert.Equal("100%", resourcesComponent.UsedVolumeString);
            Assert.Equal("100%", resourcesComponent.UsedWeightString);
        }

        [Fact]
        public void TestOversupplyGivesPartial()
        {
            ResourcesComponentCore resourcesComponent = new(
                new TestResourcesGameContent(),
                volumeCapacity: 100,
                weightCapacity: 100
            );
            resourcesComponent.CreateResources("wood", 100);
            ResourcesComponentCore targetResourcesComponent = new(
                new TestResourcesGameContent(),
                volumeCapacity: 100,
                weightCapacity: 100
            );
            targetResourcesComponent.CreateResources("wood", 50);
            Assert.Throws<ResourcesComponentCore.ResourceWeightCapacityException>(
                () => resourcesComponent.GiveResources(targetResourcesComponent, "wood", 100)
            );
            Assert.Equal("50%", resourcesComponent.UsedVolumeString);
            Assert.Equal("50%", resourcesComponent.UsedWeightString);
            Assert.Equal("100%", targetResourcesComponent.UsedVolumeString);
            Assert.Equal("100%", targetResourcesComponent.UsedWeightString);
        }

        [Fact]
        public void TestGivesAllPossible()
        {
            ResourcesComponentCore resourcesComponent = new(
                new TestResourcesGameContent(),
                volumeCapacity: 100,
                weightCapacity: 100
            );
            resourcesComponent.CreateResources("wood", 50);
            ResourcesComponentCore targetResourcesComponent = new(
                new TestResourcesGameContent(),
                volumeCapacity: 200,
                weightCapacity: 200
            );
            targetResourcesComponent.CreateResources("wood", 0);
            Assert.Throws<ResourcesComponentCore.ResourceQuantityException>(
                () => resourcesComponent.GiveResources(targetResourcesComponent, "wood", 100)
            );
            Assert.Equal("0%", resourcesComponent.UsedVolumeString);
            Assert.Equal("0%", resourcesComponent.UsedWeightString);
            Assert.Equal("25%", targetResourcesComponent.UsedVolumeString);
            Assert.Equal("25%", targetResourcesComponent.UsedWeightString);
        }
    }
}
