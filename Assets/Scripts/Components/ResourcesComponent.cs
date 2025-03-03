namespace Assets.Scripts.Components.Core
{
    using System;
    using System.Collections.Generic;
    using Assets.Scripts.Core;

    [Serializable]
    public class ResourcesComponentCore
    {
        // FIELDS //
        public uint reservedInputBufferMultiplier = 2;
        public uint reservedOuputBufferMultiplier = 4;
        public uint weightCapacity = 0;
        public uint volumeCapacity = 0;

        // When in "reserved capacity" mode, the component gains adopt new constaints.
        // Setting the reserved capacity != null actives this mode.
        // In this mode, resources can only be added or removed if they match one of
        // the reserved resources. This is useful for factories, where the I/O
        // must be managed carefully. Additionally, the reserved capacity operates
        // with a buffer value with manages how much extra capacity is reserved.
        // When the reserved capacity excedes (input value * buffer), the
        // resourced can now be released from the component.

        public Dictionary<string, uint> reservedCapacity = new();
        private GameContent GameContent;

        // PROPERTIES //

        public Dictionary<string, uint> resources = new();

        public virtual Dictionary<string, string> ResourceInfo
        {
            get
            {
                Dictionary<string, string> info = new();
                foreach (KeyValuePair<string, uint> resource in this.resources)
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
                foreach (uint resource in this.resources.Values)
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
                foreach (KeyValuePair<string, uint> resourcePair in this.resources)
                {
                    GameContent.Item item = this.GameContent.Items.GetValueOrDefault(
                        resourcePair.Key
                    );
                    if (item == null)
                    {
                        continue;
                    }
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
                foreach (KeyValuePair<string, uint> resourcePair in this.resources)
                {
                    GameContent.Item item = this.GameContent.Items.GetValueOrDefault(
                        resourcePair.Key
                    );
                    if (item == null)
                    {
                        continue;
                    }
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

        public class ResourceReservedCapacitySpaceException : ResourceException
        {
            public ResourceReservedCapacitySpaceException(string message)
                : base(message) { }
        }

        public class ResourceReservedCapacityQuantityException : ResourceException
        {
            public ResourceReservedCapacityQuantityException(string message)
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
            uint weightCapacity,
            uint volumeCapacity,
            Dictionary<string, uint> reservedCapacity = null
        )
        {
            this.GameContent = gameContent;
            this.weightCapacity = weightCapacity;
            this.volumeCapacity = volumeCapacity;
            this.reservedCapacity = reservedCapacity;
        }

        // FUNCTIONS //

        public void ForceCreateResources(string resourceName, uint amountToCreate)
        {
            // ForceCreateResources is used in situations where you want
            // to create a resource while also safely ensuring that it
            // won't accidentally get lost due to capacity constraints.
            // This is useful for factories, which can get into a state
            // where a resource is created, but can't be given to the
            // resource container because the container is full.
            // Letting the container "overfill" prevents the resouce
            // from being lost.
            uint currentResources = this.resources.GetValueOrDefault(resourceName, 0u);
            this.resources[resourceName] = currentResources + amountToCreate;
        }

        public void CreateResources(string resourceName, uint amountToCreate)
        {
            // So many null checks... @_@
            GameContent.Item item =
                this.GameContent.Items.GetValueOrDefault(
                    resourceName ?? "",
                    new GameContent.Item("")
                ) ?? new GameContent.Item("");

            uint originalAmountToCreate = amountToCreate;
            uint weightToCreate = amountToCreate * item.Weight;
            uint volumeToCreate = amountToCreate * item.Volume;
            uint currentResources = this.resources.GetValueOrDefault(resourceName, 0u);

            if (this.reservedCapacity != null && !this.reservedCapacity.ContainsKey(resourceName))
            {
                throw new ResourceReservedCapacitySpaceException(
                    $"No space reserved for {resourceName}"
                );
            }

            if (this.RemainingWeightCapacity < weightToCreate)
            {
                amountToCreate = (uint)(this.RemainingWeightCapacity / (float)item.Weight);
                this.resources[resourceName] = currentResources + amountToCreate;
                throw new ResourceWeightCapacityException(
                    $"Not enough weight capacity to create {originalAmountToCreate} {resourceName}"
                );
            }

            if (this.RemainingVolumeCapacity < volumeToCreate)
            {
                amountToCreate = (uint)(this.RemainingVolumeCapacity / (float)item.Volume);
                this.resources[resourceName] = currentResources + amountToCreate;
                throw new ResourceVolumeCapacityException(
                    $"Not enough volume capacity to create {originalAmountToCreate} {resourceName}"
                );
            }

            this.resources[resourceName] = currentResources + amountToCreate;
        }

        public void ConsumeResources(string resourceName, uint amountToConsume)
        {
            uint availableResources = this.resources.GetValueOrDefault(resourceName, 0u);
            if (availableResources < amountToConsume)
            {
                throw new ResourceException(
                    $"Does not have {amountToConsume} {resourceName} to consume"
                );
            }

            if (amountToConsume != 0)
            {
                this.resources[resourceName] -= amountToConsume;
            }
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

            if (this == target)
            {
                // Don't give resources to yourself, doing so result in resouces being magically created from nothing.
                return;
            }

            uint availableResources = this.resources.GetValueOrDefault(resourceName ?? "", (uint)0);

            GameContent.Item item =
                this.GameContent.Items.GetValueOrDefault(
                    resourceName ?? "",
                    new GameContent.Item("")
                ) ?? new GameContent.Item("");

            uint originalAmountToGive = amountToGive;
            uint weightToGive = amountToGive * item.Weight;
            uint volumeToGive = amountToGive * item.Volume;
            uint currentResources = target.resources.GetValueOrDefault(resourceName, (uint)0);

            if (availableResources == 0)
            {
                throw new ResourceQuantityException($"Does not have {resourceName} to give");
            }

            if (
                target.reservedCapacity != null
                && !target.reservedCapacity.ContainsKey(resourceName)
            )
            {
                throw new ResourceReservedCapacitySpaceException(
                    $"No space reserved for {resourceName}"
                );
            }

            // Reserved capacity maintains a minimum amount of resources in the component.
            // If we (this) have less than the reserved capacity,
            // we (this) can't give resources.
            if (
                this.reservedCapacity != null
                && this.resources.GetValueOrDefault(resourceName, 0u)
                    < this.reservedCapacity.GetValueOrDefault(resourceName, 0u)
                        * this.reservedInputBufferMultiplier
            )
            {
                throw new ResourceReservedCapacitySpaceException(
                    $"Not enough reserved capacity to recieve {originalAmountToGive} {resourceName}"
                );
            }

            // Reserved capacity maintains a maximum amount of resources in the component.
            // If they (target) have more than the reserved capacity,
            // they (target) can't recieve resources.
            if (
                target.reservedCapacity != null
                && target.resources.GetValueOrDefault(resourceName, 0u)
                    > target.reservedCapacity.GetValueOrDefault(resourceName, 0u)
                        * this.reservedOuputBufferMultiplier
            )
            {
                throw new ResourceReservedCapacitySpaceException(
                    $"Too much reserved capacity to recieve {originalAmountToGive} {resourceName}"
                );
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
                this.resources[resourceName] -= amountToGive;
                target.resources[resourceName] = currentResources + amountToGive;
                throw new ResourceWeightCapacityException(
                    $"Not enough weight capacity to give {originalAmountToGive} {resourceName}"
                );
            }

            if (target.RemainingVolumeCapacity < volumeToGive)
            {
                amountToGive = (uint)(target.RemainingVolumeCapacity / (float)item.Volume);
                this.resources[resourceName] -= amountToGive;
                target.resources[resourceName] = currentResources + amountToGive;
                throw new ResourceVolumeCapacityException(
                    $"Not enough volume capacity to give {originalAmountToGive} {resourceName}"
                );
            }

            this.resources[resourceName] -= amountToGive;
            target.resources[resourceName] = currentResources + amountToGive;
        }

        public void TakeResources(
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
            ResourcesComponentCore resourcesComponent = new(
                new TestResourcesGameContent(),
                weightCapacity: 100,
                volumeCapacity: 100
            );
            Assert.Equal((uint)0, resourcesComponent.TotalResources);
            Assert.Equal(resourcesComponent.resources.Count, 0);
            Assert.Equal(resourcesComponent.ResourceInfo.Count, 0);
            Assert.False(resourcesComponent.HasResources);
        }

        [Fact]
        public void TestResourcesOnInit()
        {
            ResourcesComponentCore resourcesComponent = new(
                new TestResourcesGameContent(),
                weightCapacity: 100,
                volumeCapacity: 100
            );
            resourcesComponent.CreateResources("wood", 10);
            resourcesComponent.CreateResources("stone", 20);
            resourcesComponent.CreateResources("iron", 30);
            Assert.Equal((uint)60, resourcesComponent.TotalResources);
            Assert.Equal(resourcesComponent.resources.Count, 3);
            Assert.Equal(resourcesComponent.ResourceInfo.Count, 3);
            Assert.True(resourcesComponent.HasResources);
        }

        [Fact]
        public void TestConsumeResources()
        {
            ResourcesComponentCore resourcesComponent = new(
                new TestResourcesGameContent(),
                weightCapacity: 100,
                volumeCapacity: 100
            );
            resourcesComponent.CreateResources("wood", 10);
            resourcesComponent.CreateResources("stone", 20);
            resourcesComponent.CreateResources("iron", 30);
            resourcesComponent.ConsumeResources("wood", 5);
            Assert.Equal((uint)55, resourcesComponent.TotalResources);
            Assert.Equal((uint)5, resourcesComponent.resources["wood"]);
        }

        [Fact]
        public void TestGiveAndTake()
        {
            ResourcesComponentCore resourcesComponent = new(
                new TestResourcesGameContent(),
                weightCapacity: 100,
                volumeCapacity: 100
            );
            resourcesComponent.CreateResources("wood", 10);
            resourcesComponent.CreateResources("stone", 20);
            resourcesComponent.CreateResources("iron", 30);

            ResourcesComponentCore targetResourcesComponent = new(
                new TestResourcesGameContent(),
                weightCapacity: 100,
                volumeCapacity: 100
            );
            targetResourcesComponent.CreateResources("wood", 5);
            targetResourcesComponent.CreateResources("stone", 10);
            targetResourcesComponent.CreateResources("iron", 15);

            resourcesComponent.GiveResources(targetResourcesComponent, "wood", 5);
            Assert.Equal((uint)55, resourcesComponent.TotalResources);
            Assert.Equal((uint)5, resourcesComponent.resources["wood"]);
            Assert.Equal((uint)35, targetResourcesComponent.TotalResources);
            Assert.Equal((uint)10, targetResourcesComponent.resources["wood"]);

            resourcesComponent.TakeResources(targetResourcesComponent, "wood", 5);
            Assert.Equal((uint)60, resourcesComponent.TotalResources);
            Assert.Equal((uint)10, resourcesComponent.resources["wood"]);
            Assert.Equal((uint)30, targetResourcesComponent.TotalResources);
            Assert.Equal((uint)5, targetResourcesComponent.resources["wood"]);
        }

        [Fact]
        public void TestTakeFromNull()
        {
            ResourcesComponentCore resourcesComponent = new(
                new TestResourcesGameContent(),
                weightCapacity: 100,
                volumeCapacity: 100
            );
            resourcesComponent.CreateResources("wood", 10);
            resourcesComponent.CreateResources("stone", 20);
            resourcesComponent.CreateResources("iron", 30);

            Assert.Throws<ResourcesComponentCore.ResourceContainerException>(
                () => resourcesComponent.TakeResources(null, "wood", 5)
            );
        }

        [Fact]
        public void TestGiveToNull()
        {
            ResourcesComponentCore resourcesComponent = new(
                new TestResourcesGameContent(),
                weightCapacity: 100,
                volumeCapacity: 100
            );
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
            ResourcesComponentCore resourcesComponent = new(
                new TestResourcesGameContent(),
                weightCapacity: 100,
                volumeCapacity: 100
            );

            ResourcesComponentCore targetResourcesComponent = new(
                new TestResourcesGameContent(),
                weightCapacity: 100,
                volumeCapacity: 100
            );
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
            ResourcesComponentCore resourcesComponent = new(
                new TestResourcesGameContent(),
                weightCapacity: 100,
                volumeCapacity: 100
            );
            resourcesComponent.CreateResources("wood", 20);

            ResourcesComponentCore targetResourcesComponent = new(
                new TestResourcesGameContent(),
                weightCapacity: 100,
                volumeCapacity: 200
            );
            targetResourcesComponent.CreateResources("wood", 90);

            Assert.Equal(90u, targetResourcesComponent.resources["wood"]);
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
            ResourcesComponentCore resourcesComponent = new(
                new TestResourcesGameContent(),
                weightCapacity: 100,
                volumeCapacity: 100
            );
            resourcesComponent.CreateResources("wood", 20);

            ResourcesComponentCore targetResourcesComponent = new(
                new TestResourcesGameContent(),
                weightCapacity: 200,
                volumeCapacity: 100
            );
            targetResourcesComponent.CreateResources("wood", 90);

            Assert.Equal(90u, targetResourcesComponent.resources["wood"]);
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
            ResourcesComponentCore resourcesComponent = new(
                new TestResourcesGameContent(),
                weightCapacity: 100,
                volumeCapacity: 100
            );
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

        [Fact]
        public void TestReservedCapacityCreate()
        {
            ResourcesComponentCore resourcesComponent = new(
                new TestResourcesGameContent(),
                volumeCapacity: 100,
                weightCapacity: 100,
                reservedCapacity: new Dictionary<string, uint> { { "wood", 100 } }
            );
            Assert.Throws<ResourcesComponentCore.ResourceReservedCapacitySpaceException>(
                () => resourcesComponent.CreateResources("stone", 100)
            );
        }

        [Fact]
        public void TestReservedCapacityGive()
        {
            ResourcesComponentCore resourcesComponent = new(
                new TestResourcesGameContent(),
                volumeCapacity: 100,
                weightCapacity: 100,
                reservedCapacity: new Dictionary<string, uint> { { "wood", 100 } }
            );
            resourcesComponent.CreateResources("wood", 100);

            ResourcesComponentCore targetResourcesComponent = new(
                new TestResourcesGameContent(),
                volumeCapacity: 100,
                weightCapacity: 100,
                reservedCapacity: new Dictionary<string, uint> { { "iron", 100 } }
            );

            Assert.Throws<ResourcesComponentCore.ResourceReservedCapacitySpaceException>(
                () => resourcesComponent.GiveResources(targetResourcesComponent, "wood", 100)
            );
        }

        [Fact]
        public void TestReservedCapacityTake()
        {
            ResourcesComponentCore resourcesComponent = new(
                new TestResourcesGameContent(),
                volumeCapacity: 200,
                weightCapacity: 200,
                reservedCapacity: new Dictionary<string, uint> { { "wood", 100 } }
            );
            resourcesComponent.CreateResources("wood", 100);

            ResourcesComponentCore targetResourcesComponent = new(
                new TestResourcesGameContent(),
                volumeCapacity: 100,
                weightCapacity: 100
            );
            targetResourcesComponent.CreateResources("wood", 100);

            resourcesComponent.TakeResources(targetResourcesComponent, "wood", 100);
        }

        [Fact]
        public void TestReservedCapacityQuantityBelowBuffer1()
        {
            // Quantity is below buffer, so we can't give resources. (1)
            // Quantity is below buffer, so we can recieve resources.

            ResourcesComponentCore resourcesComponent = new(
                new TestResourcesGameContent(),
                volumeCapacity: 1000,
                weightCapacity: 1000,
                reservedCapacity: new Dictionary<string, uint> { { "wood", 1000 } }
            );
            resourcesComponent.CreateResources("wood", 100);

            ResourcesComponentCore targetResourcesComponent = new(
                new TestResourcesGameContent(),
                volumeCapacity: 1000,
                weightCapacity: 1000
            );
            targetResourcesComponent.CreateResources("wood", 100);

            // Quantity is below buffer, so we can't give resources. (1)
            Assert.Throws<ResourcesComponentCore.ResourceReservedCapacitySpaceException>(
                () => resourcesComponent.GiveResources(targetResourcesComponent, "wood", 100)
            );
        }

        [Fact]
        public void TestReservedCapacityQuantityBelowBuffer2()
        {
            // Quantity is below buffer, so we can't give resources.
            // Quantity is below buffer, so we can recieve resources. (2)
            ResourcesComponentCore resourcesComponent = new(
                new TestResourcesGameContent(),
                volumeCapacity: 1000,
                weightCapacity: 1000,
                reservedCapacity: new Dictionary<string, uint> { { "wood", 1000 } }
            );
            resourcesComponent.CreateResources("wood", 50);

            ResourcesComponentCore targetResourcesComponent = new(
                new TestResourcesGameContent(),
                volumeCapacity: 1000,
                weightCapacity: 1000
            );
            targetResourcesComponent.CreateResources("wood", 100);

            // Quantity is below buffer, so we can recieve resources. (2)
            targetResourcesComponent.GiveResources(resourcesComponent, "wood", 100);
        }

        [Fact]
        public void TestReservedCapacityQuantityAboveBuffer3()
        {
            // Quantity is above buffer, so we can't recieve resources. (3)
            // Quantity is above buffer, so we can give resources.
            ResourcesComponentCore resourcesComponent = new(
                new TestResourcesGameContent(),
                volumeCapacity: 1000,
                weightCapacity: 1000,
                reservedCapacity: new Dictionary<string, uint> { { "wood", 10 } }
            );
            resourcesComponent.CreateResources("wood", 100);

            ResourcesComponentCore targetResourcesComponent = new(
                new TestResourcesGameContent(),
                volumeCapacity: 1000,
                weightCapacity: 1000
            );
            targetResourcesComponent.CreateResources("wood", 100);

            // Quantity is above buffer, so we can't recieve resources. (3)
            Assert.Throws<ResourcesComponentCore.ResourceReservedCapacitySpaceException>(
                () => targetResourcesComponent.GiveResources(resourcesComponent, "wood", 100)
            );
        }

        [Fact]
        public void TestReservedCapacityQuantityAboveBuffer4()
        {
            ResourcesComponentCore resourcesComponent = new(
                new TestResourcesGameContent(),
                volumeCapacity: 1000,
                weightCapacity: 1000,
                reservedCapacity: new Dictionary<string, uint> { { "wood", 10 } }
            );
            // Quantity is above buffer, so we can't recieve resources.
            // Quantity is above buffer, so we can give resources. (4)
            resourcesComponent.CreateResources("wood", 100);

            ResourcesComponentCore targetResourcesComponent = new(
                new TestResourcesGameContent(),
                volumeCapacity: 1000,
                weightCapacity: 1000
            );
            targetResourcesComponent.CreateResources("wood", 100);

            // Quantity is above buffer, so we can give resources. (4)
            resourcesComponent.GiveResources(targetResourcesComponent, "wood", 100);
        }

        [Fact]
        public void TestForceCreate()
        {
            ResourcesComponentCore resourcesComponent = new(
                new TestResourcesGameContent(),
                volumeCapacity: 1,
                weightCapacity: 1
            );
            resourcesComponent.ForceCreateResources("wood", 100);
            Assert.Equal(100u, resourcesComponent.resources["wood"]);
        }
    }
}
