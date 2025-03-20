using System.Collections.Generic;
using Assets.Scripts.Core;

namespace Assets.Scripts.Unity
{
    public class FactoryGameContent : GameContent
    {
        public override Dictionary<string, Item> Items { get; } =
            new()
            {
                // Resources
                {
                    Resources.IronOre.ToString(), //
                    new Item(Resources.IronOre.ToString(), stackSize: 100)
                },
                {
                    Resources.CopperOre.ToString(), //
                    new Item(Resources.CopperOre.ToString(), stackSize: 100)
                },
                {
                    Resources.Coal.ToString(), //
                    new Item(Resources.Coal.ToString(), stackSize: 100)
                },
                {
                    Resources.Stone.ToString(), //
                    new Item(Resources.Stone.ToString(), stackSize: 100, createFromNothing: true)
                },
                // Products
                {
                    Products.IronBars.ToString(),
                    new Item(
                        Products.IronBars.ToString(),
                        stackSize: 100,
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
                        stackSize: 100,
                        ingredients: new Dictionary<string, uint>
                        {
                            { Resources.CopperOre.ToString(), 1 },
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
                        stackSize: 10,
                        ingredients: new Dictionary<string, uint>
                        {
                            { Products.IronBars.ToString(), 4 },
                            { Resources.Stone.ToString(), 4 },
                        }
                    )
                },
                {
                    Products.Motors.ToString(),
                    new Item(
                        Products.Motors.ToString(),
                        craftTime: 5,
                        stackSize: 10,
                        ingredients: new Dictionary<string, uint>
                        {
                            { Products.IronBars.ToString(), 2 },
                            { Products.CopperBars.ToString(), 2 },
                        }
                    )
                },
                {
                    Products.Circuits.ToString(),
                    new Item(
                        Products.Circuits.ToString(),
                        craftTime: 5,
                        stackSize: 10,
                        ingredients: new Dictionary<string, uint>
                        { //
                            { Products.CopperBars.ToString(), 2 },
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
                        stackSize: 10,
                        ingredients: new Dictionary<string, uint>
                        { //
                            { Products.IronBars.ToString(), 4 },
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
                        craftTime: 40,
                        ingredients: new Dictionary<string, uint>
                        {
                            { Products.Frames.ToString(), 8 },
                            { Products.BuildingMaterials.ToString(), 4 },
                        },
                        canSpawnGameObject: true
                    )
                },
                {
                    Spawnables.CoalPlant.ToString(),
                    new Item(
                        Spawnables.CoalPlant.ToString(),
                        weight: 400,
                        volume: 200,
                        craftTime: 40,
                        ingredients: new Dictionary<string, uint>
                        {
                            { Products.Frames.ToString(), 8 },
                            { Products.BuildingMaterials.ToString(), 4 },
                            { Products.Motors.ToString(), 1 },
                        },
                        canSpawnGameObject: true
                    )
                },
                {
                    Spawnables.Factory.ToString(),
                    new Item(
                        Spawnables.Factory.ToString(),
                        weight: 400,
                        volume: 200,
                        craftTime: 40,
                        ingredients: new Dictionary<string, uint>
                        {
                            { Products.Frames.ToString(), 8 },
                            { Products.BuildingMaterials.ToString(), 4 },
                            { Products.Circuits.ToString(), 2 },
                            { Products.Motors.ToString(), 2 },
                        },
                        canSpawnGameObject: true
                    )
                },
                // Medium Buildings
                // {
                //     Spawnables.TransitHub.ToString(),
                //     new Item(
                //         Spawnables.TransitHub.ToString(),
                //         weight: 200,
                //         volume: 100,
                //         craftTime: 20,
                //         ingredients: new Dictionary<string, uint>
                //         {
                //             { Products.Frames.ToString(), 10 },
                //             { Products.BuildingMaterials.ToString(), 10 },
                //         }
                //     )
                // },
                // {
                //     Spawnables.Radar.ToString(),
                //     new Item(
                //         Spawnables.Radar.ToString(),
                //         weight: 200,
                //         volume: 100,
                //         craftTime: 20,
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
                        craftTime: 5,
                        ingredients: new Dictionary<string, uint>
                        {
                            { Products.Frames.ToString(), 4 },
                            { Products.Motors.ToString(), 1 },
                        },
                        canSpawnGameObject: true
                    )
                },
                // {
                //     Spawnables.PowerLines.ToString(),
                //     new Item(
                //         Spawnables.PowerLines.ToString(),
                //         weight: 50,
                //         craftTime: 5,
                //         canSpawnGameObject: true,
                //         createFromNothing: true
                //     )
                // },
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
            PowerLines,
        }
    }
}
