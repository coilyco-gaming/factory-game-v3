namespace Assets.Scripts.Components.Unity
{
    using System.Collections.Generic;
    using System.Linq;
    using Assets.Scripts.Unity;
    using Roy_T.AStar.Primitives;
    using UnityEngine;

    public class SpriteMapComponent : MonoBehaviour
    {
        public UnityEngine.Vector2Int _mapSize = new(20, 20);
        public System.Numerics.Vector2 MapSize => new(this._mapSize.x, this._mapSize.y);
        public Roy_T.AStar.Grids.Grid Grid { get; set; }
        public GameObject WorldGameObject { get; set; }

        private Canvas parentCanvas;
        private Dictionary<Vector2, GameObject> tiles = new();

        private GameObject MapGameObject { get; set; }
        private GameObject TilesGameObject { get; set; }

        public void Instantiate(Canvas parentCanvas)
        {
            this.parentCanvas = parentCanvas;
            this.MapGameObject = new("Map"); // I really should have drawn this in the editor
            this.MapGameObject.transform.SetParent(this.transform);
            RectTransform rectTransform = this.MapGameObject.AddComponent<RectTransform>();
            rectTransform.position = new Vector3(0, 0, 100);
            rectTransform.sizeDelta = new Vector2(0, 0);
            rectTransform.localScale = new Vector3(1, 1, 1);
            rectTransform.localPosition = new Vector3(0, 0, 0);
            rectTransform.anchorMax = new Vector2(0, 0);
            rectTransform.anchorMin = new Vector2(0, 0);
            rectTransform.pivot = new Vector2(0, 0);

            this.TilesGameObject = new("Tiles");
            this.TilesGameObject.transform.SetParent(this.MapGameObject.transform);

            this.WorldGameObject = new("World");
            this.WorldGameObject.transform.SetParent(this.MapGameObject.transform);

            float childZPosition = this.parentCanvas.planeDistance;
            UnityEngine.Vector3 childPosition = new(0, 0, childZPosition);
            this.MapGameObject.transform.position = childPosition;

            for (int x = 0; x < this.MapSize.X; x++)
            {
                for (int y = 0; y < this.MapSize.Y; y++)
                {
                    Vector2 position = new(x, y);
                    GameObject tile = this.GenerateTile(position);
                    this.tiles.Add(position, tile);
                }
            }
        }

        public Roy_T.AStar.Grids.Grid CreateGrid(GameController gameController)
        {
            // Generate base grid

            Roy_T.AStar.Grids.Grid grid =
                Roy_T.AStar.Grids.Grid.CreateGridWithLateralAndDiagonalConnections(
                    new(this._mapSize.x, this._mapSize.y),
                    new Size(Distance.FromMeters(1), Distance.FromMeters(1)),
                    Velocity.FromMetersPerSecond(1)
                );

            // Get obstacles (buildings and rocks and such)
            IEnumerable<System.Numerics.Vector2> obstacles = gameController
                .core.worldObjects.SelectMany(worldObjects => worldObjects.Value)
                // Only block pathfinding on stationary objects
                .Where(worldObject => !worldObject.Value.mobile)
                // Only block pathfinding on objects that are not pass through
                .Where(worldObject => !worldObject.Value.passThrough)
                .Select(worldObject => worldObject.Value.gridPosition);

            // Disconnect obstacles from the grid
            foreach (System.Numerics.Vector2 obstacle in obstacles)
            {
                grid.DisconnectNode(new GridPosition((int)obstacle.X, (int)obstacle.Y));
            }

            return grid;
        }

        private GameObject GenerateTile(Vector2 position)
        {
            GameObject tile = new($"Tile ({position.x}, {position.y})");
            tile.transform.SetParent(this.TilesGameObject.transform);
            tile.transform.localPosition = new Vector3(position.x, position.y, 0);
            tile.transform.localScale = new Vector3(1, 1, 1);

            SpriteRenderer sprite = tile.AddComponent<SpriteRenderer>();
            sprite.sprite = Resources.Load<Sprite>("Art/dirt");

            return tile;
        }
    }
}
