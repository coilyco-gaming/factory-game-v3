using UnityEngine;

namespace Assets.Scripts.Components
{
    public class PlayerComponent : MonoBehaviour
    {
        public int maxZoom = 10;
        public GameObject backgroundFog;
        private int xSize;
        private int ySize;

        public void Instantiate(int xSize, int ySize)
        {
            this.xSize = xSize;
            this.ySize = ySize;
            this.Reset();
        }

        public void Reset()
        {
            Camera.main.transform.position = new Vector3(
                this.xSize / 2,
                this.ySize / 2,
                Camera.main.transform.position.z
            );
        }

        public void Update()
        {
            if (Input.GetKeyDown(KeyCode.W))
            {
                Camera.main.transform.position += Vector3.up;
            }
            if (Input.GetKeyDown(KeyCode.A))
            {
                Camera.main.transform.position += Vector3.left;
            }
            if (Input.GetKeyDown(KeyCode.S))
            {
                Camera.main.transform.position += Vector3.down;
            }
            if (Input.GetKeyDown(KeyCode.D))
            {
                Camera.main.transform.position += Vector3.right;
            }
            if (Input.mouseScrollDelta.y > 0 && Camera.main.orthographicSize > 1)
            {
                Camera.main.orthographicSize--;
            }
            if (Input.mouseScrollDelta.y < 0 && Camera.main.orthographicSize < this.maxZoom)
            {
                Camera.main.orthographicSize++;
            }
        }

        public System.Numerics.Vector2 GetGridPosition()
        {
            return new System.Numerics.Vector2(
                Camera.main.transform.position.x,
                Camera.main.transform.position.y
            );
        }

        public void ToggleFogPosition(bool close)
        {
            this.backgroundFog.transform.position = close
                ? new Vector3(0, 0, 50)
                : new Vector3(0, 0, 300);
        }
    }
}
