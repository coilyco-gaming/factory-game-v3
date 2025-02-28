using System.Collections.Generic;
using Assets.Scripts.Core;

namespace Assets.Scripts.Unity
{
    public class FactoryGameContent : GameContent
    {
        public override Dictionary<string, Item> Items { get; } =
            new()
            {
                { Resources.Iron.ToString(), new Item(Resources.Iron.ToString()) },
                { Resources.Coal.ToString(), new Item(Resources.Coal.ToString()) },
                { Resources.Copper.ToString(), new Item(Resources.Copper.ToString()) },
                { Resources.Stone.ToString(), new Item(Resources.Stone.ToString()) },
                {
                    Products.BuildingMaterials.ToString(),
                    new Item(
                        Products.BuildingMaterials.ToString(),
                        weight: 20,
                        volume: 5,
                        craftTime: 5,
                        ingredients: new Dictionary<string, uint>
                        {
                            { Resources.Iron.ToString(), 10 },
                            { Resources.Stone.ToString(), 10 },
                        }
                    )
                },
                {
                    Products.Motors.ToString(),
                    new Item(
                        Products.Motors.ToString(),
                        craftTime: 5,
                        ingredients: new Dictionary<string, uint>
                        {
                            { Resources.Iron.ToString(), 5 },
                            { Resources.Copper.ToString(), 5 },
                        }
                    )
                },
                {
                    Products.Circuits.ToString(),
                    new Item(
                        Products.Circuits.ToString(),
                        craftTime: 5,
                        ingredients: new Dictionary<string, uint>
                        { //
                            { Resources.Copper.ToString(), 5 },
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
                        ingredients: new Dictionary<string, uint>
                        { //
                            { Resources.Iron.ToString(), 10 },
                        }
                    )
                },
                {
                    Spawnables.Warehouse.ToString(),
                    new Item(
                        Spawnables.Warehouse.ToString(),
                        weight: 300,
                        volume: 150,
                        craftTime: 100,
                        ingredients: new Dictionary<string, uint>
                        {
                            { Products.Frames.ToString(), 10 },
                            { Products.BuildingMaterials.ToString(), 10 },
                        }
                    )
                },
                {
                    Spawnables.Radar.ToString(),
                    new Item(
                        Spawnables.Radar.ToString(),
                        weight: 150,
                        volume: 75,
                        craftTime: 100,
                        ingredients: new Dictionary<string, uint>
                        {
                            { Products.Frames.ToString(), 5 },
                            { Products.BuildingMaterials.ToString(), 5 },
                            { Products.Circuits.ToString(), 5 },
                        }
                    )
                },
                {
                    Spawnables.CoalPlant.ToString(),
                    new Item(
                        Spawnables.CoalPlant.ToString(),
                        weight: 300,
                        volume: 150,
                        craftTime: 100,
                        ingredients: new Dictionary<string, uint>
                        {
                            { Products.Frames.ToString(), 10 },
                            { Products.BuildingMaterials.ToString(), 10 },
                            { Products.Motors.ToString(), 5 },
                        }
                    )
                },
                {
                    Spawnables.Mine.ToString(),
                    new Item(
                        Spawnables.Mine.ToString(),
                        weight: 50,
                        volume: 50,
                        craftTime: 100,
                        ingredients: new Dictionary<string, uint>
                        {
                            { Products.Frames.ToString(), 10 },
                            { Products.Motors.ToString(), 1 },
                        }
                    )
                },
                {
                    Spawnables.Factory.ToString(),
                    new Item(
                        Spawnables.Factory.ToString(),
                        weight: 300,
                        volume: 150,
                        craftTime: 100,
                        ingredients: new Dictionary<string, uint>
                        {
                            { Products.Frames.ToString(), 10 },
                            { Products.BuildingMaterials.ToString(), 10 },
                            { Products.Circuits.ToString(), 5 },
                            { Products.Motors.ToString(), 5 },
                        }
                    )
                },
            };

        public enum Resources
        {
            Iron,
            Coal,
            Copper,
            Stone,
        }

        public enum Products
        {
            BuildingMaterials,
            Motors,
            Circuits,
            Frames,
        }

        public enum Spawnables
        {
            Radar,
            CoalPlant,
            Mine,
            Factory,
            Warehouse,
        }
    }
}
