using System.Collections.Generic;
using Assets.Scripts.Core;

namespace Assets.Scripts.Unity
{
    public class FactoryGameContent : GameContent
    {
        public override Dictionary<string, Object> Objects { get; } = new() { };

        public override Dictionary<string, Item> Items { get; } =
            new()
            {
                // Resources
                {
                    Resources.IronOre.ToString(), //
                    new Item(Resources.IronOre.ToString(), stackSize: 200)
                },
                {
                    Resources.CopperOre.ToString(),
                    new Item(Resources.CopperOre.ToString(), stackSize: 200)
                },
                {
                    Resources.Coal.ToString(), //
                    new Item(Resources.Coal.ToString(), stackSize: 200)
                },
                {
                    Resources.Stone.ToString(),
                    new Item(Resources.Stone.ToString(), stackSize: 200)
                },
                // Products
                {
                    Products.IronBars.ToString(),
                    new Item(
                        Products.IronBars.ToString(),
                        stackSize: 200,
                        ingredients: new Dictionary<string, uint>
                        {
                            { Resources.IronOre.ToString(), 1 },
                        }
                    )
                },
                {
                    Products.CopperBars.ToString(),
                    new Item(
                        Products.CopperBars.ToString(),
                        stackSize: 200,
                        ingredients: new Dictionary<string, uint>
                        {
                            { Resources.IronOre.ToString(), 1 },
                        }
                    )
                },
                {
                    Products.BuildingMaterials.ToString(),
                    new Item(
                        Products.BuildingMaterials.ToString(),
                        weight: 20,
                        volume: 5,
                        craftTime: 5,
                        stackSize: 20,
                        ingredients: new Dictionary<string, uint>
                        {
                            { Products.IronBars.ToString(), 10 },
                            { Resources.Stone.ToString(), 10 },
                        }
                    )
                },
                {
                    Products.Motors.ToString(),
                    new Item(
                        Products.Motors.ToString(),
                        craftTime: 5,
                        stackSize: 20,
                        ingredients: new Dictionary<string, uint>
                        {
                            { Products.IronBars.ToString(), 5 },
                            { Products.CopperBars.ToString(), 5 },
                        }
                    )
                },
                {
                    Products.Circuits.ToString(),
                    new Item(
                        Products.Circuits.ToString(),
                        craftTime: 5,
                        stackSize: 20,
                        ingredients: new Dictionary<string, uint>
                        { //
                            { Products.CopperBars.ToString(), 5 },
                        }
                    )
                },
                {
                    Products.Frames.ToString(),
                    new Item(
                        Products.Frames.ToString(),
                        weight: 10,
                        volume: 10,
                        craftTime: 5,
                        stackSize: 20,
                        ingredients: new Dictionary<string, uint>
                        { //
                            { Products.IronBars.ToString(), 10 },
                        }
                    )
                },
                // Large Buildings
                {
                    Spawnables.StorageWarehouse.ToString(),
                    new Item(
                        Spawnables.StorageWarehouse.ToString(),
                        weight: 400,
                        volume: 200,
                        craftTime: 50,
                        ingredients: new Dictionary<string, uint>
                        {
                            { Products.Frames.ToString(), 10 },
                            { Products.BuildingMaterials.ToString(), 10 },
                        }
                    )
                },
                {
                    Spawnables.CoalPlant.ToString(),
                    new Item(
                        Spawnables.CoalPlant.ToString(),
                        weight: 400,
                        volume: 200,
                        craftTime: 50,
                        ingredients: new Dictionary<string, uint>
                        {
                            { Products.Frames.ToString(), 10 },
                            { Products.BuildingMaterials.ToString(), 10 },
                            { Products.Motors.ToString(), 5 },
                        }
                    )
                },
                {
                    Spawnables.Factory.ToString(),
                    new Item(
                        Spawnables.Factory.ToString(),
                        weight: 400,
                        volume: 200,
                        craftTime: 50,
                        ingredients: new Dictionary<string, uint>
                        {
                            { Products.Frames.ToString(), 10 },
                            { Products.BuildingMaterials.ToString(), 10 },
                            { Products.Circuits.ToString(), 5 },
                            { Products.Motors.ToString(), 5 },
                        }
                    )
                },
                // Medium Buildings
                {
                    Spawnables.TransitHub.ToString(),
                    new Item(
                        Spawnables.TransitHub.ToString(),
                        weight: 200,
                        volume: 100,
                        craftTime: 50,
                        ingredients: new Dictionary<string, uint>
                        {
                            { Products.Frames.ToString(), 10 },
                            { Products.BuildingMaterials.ToString(), 10 },
                        }
                    )
                },
                // {
                //     Spawnables.Radar.ToString(),
                //     new Item(
                //         Spawnables.Radar.ToString(),
                //         weight: 200,
                //         volume: 100,
                //         craftTime: 50,
                //         ingredients: new Dictionary<string, uint>
                //         {
                //             { Products.Frames.ToString(), 5 },
                //             { Products.BuildingMaterials.ToString(), 5 },
                //             { Products.Circuits.ToString(), 5 },
                //         }
                //     )
                // },
                // Small Buildings
                {
                    Spawnables.MiningDrill.ToString(),
                    new Item(
                        Spawnables.MiningDrill.ToString(),
                        weight: 50,
                        volume: 50,
                        craftTime: 50,
                        ingredients: new Dictionary<string, uint>
                        {
                            { Products.Frames.ToString(), 5 },
                            { Products.Motors.ToString(), 1 },
                        }
                    )
                },
            };

        public enum Resources
        {
            IronOre,
            CopperOre,
            Coal,
            Stone,
        }

        public enum Products
        {
            IronBars,
            CopperBars,
            BuildingMaterials,
            Motors,
            Circuits,
            Frames,
        }

        public enum Spawnables
        {
            Truck,
            CoalPlant,
            MiningDrill,
            Factory,
            StorageWarehouse,
            TransitHub,
        }

        public enum DispatchVerbs
        {
            Deploy,
            Retrieve,
        }
    }
}
